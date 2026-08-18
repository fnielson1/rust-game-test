//! The JSON level format and the loader that turns it into a [`LevelAsset`].
//!
//! A level is a list of segments. Each segment is two endpoints and a thickness; its angle
//! and length are whatever the endpoints imply, and a gap is simply the absence of a segment
//! between two others. That is the whole format -- there is no explicit gap, angle, or spacer
//! entry, because none is needed.
//!
//! A segment may also carry a `control` point, which bends it into a curve; see
//! [`crate::levels::curve`] for what that means geometrically and how it is built. This file owns
//! the *format*: which fields exist and which combinations are rejected. The geometry those fields
//! imply lives next door.
//!
//! Files are registered under the `.level.json` extension rather than plain `.json` so that any
//! other JSON the project loads later can't accidentally be routed through this loader. Bevy
//! matches on the full extension (everything after the first `.` in the file name), so
//! `level1.level.json` resolves to `level.json`.

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::{Quat, Transform, TypePath, Vec2};
use serde::Deserialize;

/// One segment of level geometry: a straight line, or a curve if it declares a `control` point.
///
/// `start` and `end` are world-space points. The surface built from them is centered on the
/// segment and extends `thickness / 2.0` to either side, so `thickness` is measured perpendicular
/// to the segment, not vertically.
///
/// Both optional fields default to `None`, which is what keeps files written before curves existed
/// valid: a segment with no `control` is the straight line it always was, built by the same code
/// path and producing the same entity.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct Segment {
  pub start: [f32; 2],
  pub end: [f32; 2],
  pub thickness: f32,
  /// Bends the segment into a quadratic Bezier curve pulled toward this point.
  ///
  /// The curve passes through `start` and `end` but *not* through `control` -- it peaks around
  /// halfway toward it. Put the point above the line for a hill, below for a vale, and off-center
  /// for a lopsided one.
  ///
  /// Quadratic specifically: such a curve is always a simple convex arc, so it cannot
  /// self-intersect, cusp, or inflect. Every curve that can be written here is therefore a
  /// well-formed hill or vale, which is what keeps the offset geometry in
  /// [`crate::levels::curve`] tractable.
  #[serde(default)]
  pub control: Option<[f32; 2]>,
  /// Overrides how many straight pieces the curve is approximated by.
  ///
  /// Normally unnecessary -- the count is derived from the curve's own size so that a small curve
  /// and a large one come out equally smooth. Set it to a small number for a deliberately faceted
  /// surface. Ignored by straight segments, which are never subdivided.
  #[serde(default)]
  pub subdivisions: Option<u32>,
}

/// Why a segment can't produce a surface.
///
/// These are authoring mistakes in a hand-edited file, not engine failures: the level still
/// loads, the offending segment is skipped, and this says which mistake it was.
#[derive(Debug)]
pub enum SegmentError {
  /// Endpoints coincide, or a coordinate is NaN/infinite. `atan2(0, 0)` is 0 and a zero-area
  /// rectangle is a degenerate collider that avian accepts and then behaves oddly around, so
  /// this is rejected rather than clamped.
  InvalidLength(f32),
  /// Thickness is zero, negative, or non-finite -- same degenerate-collider problem.
  InvalidThickness(f32),
  /// A control-point coordinate is NaN or infinite. Same reasoning as the endpoint checks: every
  /// sample along the curve would inherit it, and the segment would spawn as an invisible entity
  /// at an undefined position.
  NonFiniteControl([f32; 2]),
  /// The curve bends tighter than half its own thickness. Below that radius the inner edge of the
  /// surface crosses itself, the pieces there turn inside out, and the resulting collider is
  /// wrong in a way that is nearly impossible to spot on screen -- so this is rejected rather
  /// than drawn.
  TooSharpForThickness { radius: f32, thickness: f32 },
  /// The control point lies on the line through the endpoints but outside them, folding the curve
  /// back along itself. The fold point has no tangent, so there is no direction to offset the
  /// surface in.
  FoldedCurve,
}

impl core::fmt::Display for SegmentError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::InvalidLength(length) => {
        write!(f, "endpoints must differ (length was {length})")
      }
      Self::InvalidThickness(thickness) => {
        write!(f, "thickness must be positive (was {thickness})")
      }
      Self::NonFiniteControl([x, y]) => {
        write!(f, "control point must be finite (was [{x}, {y}])")
      }
      Self::TooSharpForThickness { radius, thickness } => {
        write!(
          f,
          "curve bends too sharply for its thickness: tightest radius {radius} is under half of \
           thickness {thickness} -- make it thinner or bend it less"
        )
      }
      Self::FoldedCurve => {
        write!(
          f,
          "control point folds the curve back on itself -- move it off the line through the \
           endpoints, or between them"
        )
      }
    }
  }
}

impl Segment {
  /// Whether this segment bends. Straight segments take a simpler and entirely separate spawn
  /// path, so this is the one place that decision is made.
  pub fn is_curved(&self) -> bool {
    self.control.is_some()
  }

  /// Distance between the endpoints. For a straight segment this is the length of the rectangle
  /// built from it; for a curved one it is only the chord, and says nothing about arc length.
  pub fn length(&self) -> f32 {
    Vec2::from(self.end).distance(Vec2::from(self.start))
  }

  /// Rejects segments that would produce a degenerate surface.
  ///
  /// The `is_finite` checks matter because NaN fails every ordered comparison: a bare
  /// `length <= 0.0` would wave a NaN coordinate through and spawn an invisible entity at an
  /// undefined position.
  ///
  /// Ordered cheapest-first, and finiteness before geometry: the curvature check below reads every
  /// control point, so a non-finite one has to be caught before it gets there.
  pub fn validate(&self) -> Result<(), SegmentError> {
    if !self.thickness.is_finite() || self.thickness <= 0.0 {
      return Err(SegmentError::InvalidThickness(self.thickness));
    }
    if let Some(control) = self.control
      && !Vec2::from(control).is_finite()
    {
      return Err(SegmentError::NonFiniteControl(control));
    }
    let length = self.length();
    if !length.is_finite() || length <= 0.0 {
      return Err(SegmentError::InvalidLength(length));
    }
    self.validate_curvature()
  }

  /// Places a `length x thickness` rectangle along this segment: centered on the midpoint of
  /// the two endpoints, rotated to the angle between them.
  ///
  /// Declaring a segment right-to-left yields the same surface rotated a half turn, which for a
  /// rectangle occupies exactly the same space.
  pub fn transform(&self, z: f32) -> Transform {
    let start = Vec2::from(self.start);
    let end = Vec2::from(self.end);
    let midpoint = (start + end) / 2.0;
    let delta = end - start;
    Transform::from_xyz(midpoint.x, midpoint.y, z)
      .with_rotation(Quat::from_rotation_z(delta.y.atan2(delta.x)))
  }
}

/// A parsed level file.
///
/// This is both the `serde` target and the Bevy [`Asset`]; there's no separate intermediate
/// type, since the on-disk shape and the in-memory shape are the same list.
///
/// `deny_unknown_fields` is deliberate: these files are hand-edited, and a silently-ignored
/// misspelled key produces a level that is subtly wrong rather than one that fails loudly.
#[derive(Asset, TypePath, Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct LevelAsset {
  pub segments: Vec<Segment>,
}

/// Why a level file failed to load.
///
/// Hand-written rather than derived: `thiserror` isn't a dependency of this crate, and the two
/// variants don't justify adding one. [`bevy::prelude::BevyError`] accepts anything that converts
/// into `Box<dyn Error + Send + Sync>`, which a plain [`core::error::Error`] impl satisfies.
#[derive(Debug)]
pub enum LevelLoaderError {
  /// The file couldn't be read at all.
  Io(std::io::Error),
  /// The bytes were read but didn't parse as a level. Carries the path so the log line names
  /// the offending file, which the raw `serde_json` error does not.
  Parse {
    path: String,
    source: serde_json::Error,
  },
}

impl core::fmt::Display for LevelLoaderError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Io(error) => write!(f, "could not read level file: {error}"),
      Self::Parse { path, source } => {
        write!(f, "could not parse level file `{path}`: {source}")
      }
    }
  }
}

impl core::error::Error for LevelLoaderError {
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    match self {
      Self::Io(error) => Some(error),
      Self::Parse { source, .. } => Some(source),
    }
  }
}

impl From<std::io::Error> for LevelLoaderError {
  fn from(error: std::io::Error) -> Self {
    Self::Io(error)
  }
}

/// Reads `.level.json` files into [`LevelAsset`]s.
///
/// `TypePath` is required by the [`AssetLoader`] trait itself, not just by the asset.
#[derive(Default, TypePath)]
pub struct LevelLoader;

impl AssetLoader for LevelLoader {
  type Asset = LevelAsset;
  type Settings = ();
  type Error = LevelLoaderError;

  async fn load(
    &self,
    reader: &mut dyn Reader,
    _settings: &(),
    load_context: &mut LoadContext<'_>,
  ) -> Result<Self::Asset, Self::Error> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await?;
    serde_json::from_slice(&bytes).map_err(|source| LevelLoaderError::Parse {
      path: load_context.path().to_string(),
      source,
    })
  }

  fn extensions(&self) -> &[&str] {
    &["level.json"]
  }
}
