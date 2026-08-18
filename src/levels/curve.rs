//! Turning a curved [`Segment`] into geometry: where to sample it, and how to build the ribbon of
//! surface that follows it.
//!
//! A curved segment is a quadratic Bezier. Nothing in the engine draws or collides with a Bezier
//! directly, so the curve is *flattened* -- approximated by a chain of short straight pieces --
//! and the mesh and the collider are both built from that same chain. Building both from one set
//! of points is the whole trick: the surface the player collides with is the surface drawn on
//! screen, so there is no outward notch at a joint for a rolling ball to catch on.
//!
//! Straight segments never come through here. They keep the rectangle path in
//! [`crate::levels::spawn_level`], which is cheaper and already correct.
//!
//! The math leans on two properties of quadratics specifically, both of which cubics lack:
//!
//! - The second derivative is constant, which turns "how many pieces do I need?" into a closed
//!   form instead of a recursive subdivision loop that would re-run on every level rebuild.
//! - The cross product of the first and second derivatives is also constant, so the tightest
//!   radius of curvature has an exact answer rather than a sampled estimate.

use crate::levels::level_asset::{Segment, SegmentError};
use avian2d::prelude::Collider;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::prelude::{Transform, Vec2};

/// How far the flattened chain is allowed to stray from the true curve, in world units.
///
/// This is what "smooth" means here, and it is measured in world space rather than in pieces, so
/// a small curve and a large one come out equally smooth without the author saying anything. Half
/// a world unit is well under a pixel at normal zoom; lower it if facets ever become visible.
const TOLERANCE: f32 = 0.5;

/// Floor on the piece count. One piece is a straight chord, which is the right answer for a curve
/// whose control point happens to be collinear.
const MIN_SUBDIVISIONS: u32 = 1;

/// Ceiling on the piece count, so a wild control point can't ask for thousands of colliders on a
/// path that re-runs every time the level file is saved.
const MAX_SUBDIVISIONS: u32 = 64;

/// Speed (world units per unit `t`) below which the curve is treated as folding back on itself.
///
/// A quadratic's speed only reaches zero at a cusp, which needs a collinear control point outside
/// the endpoints. There is no tangent there, so there is no direction to offset the surface in.
const CUSP_SPEED: f32 = 1e-3;

impl Segment {
  /// The three Bezier control points, with a straight segment folded in as the degenerate case.
  ///
  /// Using the midpoint for a missing `control` is not a fudge: a quadratic whose control point is
  /// the midpoint of its endpoints *is* the straight line between them, exactly. That makes every
  /// function below correct for straight segments too, which is what lets the tests compare a
  /// curve against the straight case without a second code path.
  fn bezier(&self) -> (Vec2, Vec2, Vec2) {
    let start = Vec2::from(self.start);
    let end = Vec2::from(self.end);
    let control = self.control.map_or((start + end) / 2.0, Vec2::from);
    (start, control, end)
  }

  /// The two legs of the control polygon: `start -> control` and `control -> end`.
  ///
  /// Most of the math below is easier in terms of these than the raw points -- the first
  /// derivative is a lerp between them, and the second is their difference.
  fn legs(&self) -> (Vec2, Vec2) {
    let (start, control, end) = self.bezier();
    (control - start, end - control)
  }

  /// How many straight pieces to approximate this curve with.
  ///
  /// A quadratic's second derivative is the constant `2 * (P0 - 2*P1 + P2)`, so splitting the
  /// curve into `n` equal steps in `t` bounds the gap between the curve and its chords at
  /// `|P0 - 2*P1 + P2| / (4 * n^2)`. Inverting that for [`TOLERANCE`] gives the count outright.
  ///
  /// A declared `subdivisions` overrides the computation entirely -- it is an instruction, not a
  /// hint -- but is still clamped, since the bounds exist to protect the rebuild path rather than
  /// to second-guess the author.
  pub fn subdivision_count(&self) -> u32 {
    if let Some(declared) = self.subdivisions {
      return declared.clamp(MIN_SUBDIVISIONS, MAX_SUBDIVISIONS);
    }
    let (leg_in, leg_out) = self.legs();
    let second_derivative = (leg_out - leg_in).length();
    let exact = (second_derivative / (4.0 * TOLERANCE)).sqrt().ceil();
    // `as` saturates at the bounds of `u32` rather than wrapping, and `validate` has already
    // rejected non-finite coordinates, so the clamp here is a floor and ceiling -- not a guard
    // against the cast producing something absurd.
    (exact as u32).clamp(MIN_SUBDIVISIONS, MAX_SUBDIVISIONS)
  }

  /// Point on the curve at `t`, where `t` runs 0 at `start` to 1 at `end`.
  fn point_at(&self, t: f32) -> Vec2 {
    let (start, control, end) = self.bezier();
    let inverse = 1.0 - t;
    inverse * inverse * start + 2.0 * inverse * t * control + t * t * end
  }

  /// First derivative at `t`: the direction of travel along the curve, scaled by speed.
  ///
  /// Not normalized, because the magnitude is the speed and two callers below need it.
  fn tangent_at(&self, t: f32) -> Vec2 {
    let (leg_in, leg_out) = self.legs();
    2.0 * (leg_in.lerp(leg_out, t))
  }

  /// The flattened chain: `subdivision_count() + 1` points at uniform steps in `t`.
  ///
  /// A straight segment yields exactly its two endpoints, since its second derivative is zero and
  /// the count therefore bottoms out at [`MIN_SUBDIVISIONS`].
  pub fn sample_points(&self) -> Vec<Vec2> {
    let count = self.subdivision_count();
    (0..=count)
      .map(|step| self.point_at(step as f32 / count as f32))
      .collect()
  }

  /// The smallest speed anywhere on the curve, and where along the control polygon it occurs.
  ///
  /// The first derivative is a lerp between the two legs, so its magnitude is minimized by the
  /// usual point-to-line projection -- clamped to `[0, 1]` because only that span is on the curve.
  fn min_speed(&self) -> f32 {
    let (leg_in, leg_out) = self.legs();
    let spread = leg_out - leg_in;
    let closest = if spread.length_squared() > 0.0 {
      (-leg_in.dot(spread) / spread.length_squared()).clamp(0.0, 1.0)
    } else {
      // Both legs equal: speed is constant, so any `t` reports it.
      0.0
    };
    2.0 * leg_in.lerp(leg_out, closest).length()
  }

  /// Rejects a curve whose shape can't carry its own thickness.
  ///
  /// The tightest radius has an exact answer here. Curvature is
  /// `|B' x B''| / |B'|^3`, and for a quadratic the numerator works out to the constant
  /// `4 * (leg_in x leg_out)` -- so the radius is smallest exactly where the speed is, and
  /// [`Self::min_speed`] already found that point.
  ///
  /// Below `thickness / 2.0` the inner edge of the ribbon crosses itself and the pieces there turn
  /// inside out. Called from `Segment::validate`, so this rides the same skip-and-warn path as
  /// every other authoring mistake.
  pub(super) fn validate_curvature(&self) -> Result<(), SegmentError> {
    let speed = self.min_speed();
    if speed <= CUSP_SPEED {
      return Err(SegmentError::FoldedCurve);
    }
    let (leg_in, leg_out) = self.legs();
    let cross = leg_in.perp_dot(leg_out).abs();
    if cross == 0.0 {
      // Collinear and not folded: a straight line, of infinite radius. Nothing to check.
      return Ok(());
    }
    let radius = speed.powi(3) / (4.0 * cross);
    if radius < self.thickness / 2.0 {
      return Err(SegmentError::TooSharpForThickness {
        radius,
        thickness: self.thickness,
      });
    }
    Ok(())
  }

  /// Builds the band of surface following this curve.
  pub fn ribbon(&self) -> Ribbon {
    let count = self.subdivision_count();
    let half = self.thickness / 2.0;

    let points = self.sample_points();
    let offsets: Vec<Vec2> = (0..=count)
      .map(|step| {
        let t = step as f32 / count as f32;
        // `perp` is the quarter-turn to the left, so `left` below really is the left-hand side
        // looking from `start` toward `end`. `normalize_or_zero` can only fire on a cusp, which
        // `validate_curvature` has already rejected; it is here so a caller that skipped
        // validation gets a rejected collider rather than a mesh full of NaN.
        self.tangent_at(t).perp().normalize_or_zero() * half
      })
      .collect();

    let centroid = points.iter().copied().sum::<Vec2>() / points.len() as f32;
    let rails = points
      .iter()
      .zip(&offsets)
      .map(|(point, offset)| {
        let local = *point - centroid;
        [local + *offset, local - *offset]
      })
      .collect();

    Ribbon { centroid, rails }
  }
}

/// A band of surface following a curve, ready to become a mesh and a collider.
///
/// Stored as one entry per sample point rather than one per piece, and that is the point: two
/// neighbouring pieces read the *same* entry for their shared edge, so they meet exactly. Storing
/// four corners per piece would let rounding drift the two copies apart and open a seam.
pub struct Ribbon {
  /// Center of the sampled curve. The entity's translation, and the origin the points in `rails`
  /// are measured from -- world-space vertices with an origin transform would work too, but would
  /// give every curve a bounding volume spanning the level.
  pub centroid: Vec2,
  /// Per sample point, the pair of offset points either side of the curve, in local space:
  /// `[left, right]` looking from `start` toward `end`.
  rails: Vec<[Vec2; 2]>,
}

impl Ribbon {
  /// Number of quads in the band: one fewer than the number of sample points.
  pub fn quad_count(&self) -> usize {
    self.rails.len().saturating_sub(1)
  }

  /// The four corners of one quad, wound counter-clockwise.
  ///
  /// Both the mesh and the collider read this, which is what guarantees the drawn surface and the
  /// collided surface are the same surface.
  pub fn quad(&self, index: usize) -> [Vec2; 4] {
    let [left_near, right_near] = self.rails[index];
    let [left_far, right_far] = self.rails[index + 1];
    [left_near, right_near, right_far, left_far]
  }

  /// The entity transform: a plain translation, since a curve has no single meaningful angle.
  pub fn transform(&self, z: f32) -> Transform {
    Transform::from_xyz(self.centroid.x, self.centroid.y, z)
  }

  /// The drawn band: two triangles per quad, over shared vertices so there are no seams.
  ///
  /// All three of position, UV, and normal are supplied even though a flat 2D band barely needs
  /// the last two. The 2D pipeline specializes on the attributes a mesh actually carries, and a
  /// layout it doesn't expect shows up as an invisible or garbled surface rather than an error --
  /// a bad thing to debug for the sake of two `vec![]`s.
  pub fn mesh(&self) -> Mesh {
    let mut positions = Vec::with_capacity(self.rails.len() * 2);
    let mut uvs = Vec::with_capacity(self.rails.len() * 2);
    let spans = self.quad_count().max(1) as f32;
    for (index, [left, right]) in self.rails.iter().enumerate() {
      positions.push([left.x, left.y, 0.0]);
      positions.push([right.x, right.y, 0.0]);
      let along = index as f32 / spans;
      uvs.push([along, 0.0]);
      uvs.push([along, 1.0]);
    }

    let mut indices = Vec::with_capacity(self.quad_count() * 6);
    for quad in 0..self.quad_count() as u32 {
      // Vertices were pushed in left/right pairs, so quad `n` owns 2n..2n+4.
      let base = quad * 2;
      indices.extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
    }

    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
      .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
      .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
      .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
      .with_inserted_indices(Indices::U32(indices))
  }

  /// The collided band: one convex piece per quad, from the same corners the mesh draws.
  ///
  /// Convex pieces rather than a trimesh or a polyline over the whole band. A trimesh is concave
  /// and its internal shared edges are the classic source of ghost collisions for a body sliding
  /// across them; a polyline has no thickness for a fast body to be stopped by. A compound of
  /// convex quads has neither problem, and is what avian's solver handles best.
  ///
  /// `None` means a quad was too degenerate to hull -- reported by the caller as a skipped
  /// segment, the same as any other authoring mistake.
  pub fn collider(&self) -> Option<Collider> {
    if self.quad_count() == 0 {
      return None;
    }
    let pieces = (0..self.quad_count())
      .map(|index| {
        Collider::convex_hull(self.quad(index).to_vec()).map(|hull| (Vec2::ZERO, 0.0, hull))
      })
      .collect::<Option<Vec<_>>>()?;
    Some(Collider::compound(pieces))
  }
}

#[cfg(test)]
mod tests {
  use super::{MAX_SUBDIVISIONS, MIN_SUBDIVISIONS, TOLERANCE};
  use crate::levels::level_asset::{Segment, SegmentError};
  use bevy::prelude::Vec2;

  /// A gentle symmetric hill: 300 wide, control point 105 above the line.
  fn hill() -> Segment {
    Segment {
      start: [-150.0, 0.0],
      end: [150.0, 0.0],
      thickness: 30.0,
      control: Some([0.0, 105.0]),
      subdivisions: None,
    }
  }

  fn straight() -> Segment {
    Segment {
      start: [-150.0, 0.0],
      end: [150.0, 0.0],
      thickness: 30.0,
      control: None,
      subdivisions: None,
    }
  }

  #[test]
  fn straight_segment_samples_only_its_endpoints() {
    let points = straight().sample_points();
    assert_eq!(points, vec![Vec2::new(-150.0, 0.0), Vec2::new(150.0, 0.0)]);
  }

  #[test]
  fn curve_peaks_at_half_the_control_offset() {
    let segment = hill();
    // A quadratic reaches 0.25*P0 + 0.5*P1 + 0.25*P2 at t = 0.5, which for a symmetric hill is
    // exactly half the control point's offset.
    let apex = segment.point_at(0.5).y;
    assert!((apex - 52.5).abs() < 0.01, "apex at {apex}, expected 52.5");

    // The sampled chain need not include t = 0.5 -- the derived count here is odd -- so its peak
    // is held to the flattening tolerance rather than to the apex exactly. That is the property
    // worth asserting anyway: however the count falls out, the chain stays within TOLERANCE.
    let peak = segment
      .sample_points()
      .iter()
      .map(|point| point.y)
      .fold(f32::MIN, f32::max);
    assert!(
      (peak - apex).abs() < TOLERANCE,
      "sampled peak {peak} strayed from apex {apex} by more than {TOLERANCE}"
    );
  }

  #[test]
  fn curve_passes_through_its_endpoints_but_not_its_control_point() {
    let points = hill().sample_points();
    assert_eq!(points.first().copied(), Some(Vec2::new(-150.0, 0.0)));
    assert_eq!(points.last().copied(), Some(Vec2::new(150.0, 0.0)));
    assert!(!points.contains(&Vec2::new(0.0, 105.0)));
  }

  #[test]
  fn swapping_endpoints_reverses_the_same_points() {
    let forward = hill().sample_points();
    let mut swapped = hill();
    swapped.start = hill().end;
    swapped.end = hill().start;
    let backward = swapped.sample_points();

    assert_eq!(forward.len(), backward.len());
    for (ahead, behind) in forward.iter().zip(backward.iter().rev()) {
      assert!(ahead.distance(*behind) < 0.001, "{ahead} vs {behind}");
    }
  }

  #[test]
  fn collinear_control_point_yields_a_straight_line() {
    let mut collinear = straight();
    collinear.control = Some([0.0, 0.0]);
    for point in collinear.sample_points() {
      assert!(point.y.abs() < 0.001, "strayed off the line at {point}");
    }
    assert!(collinear.validate_curvature().is_ok());
  }

  #[test]
  fn subdivision_count_rises_with_curve_size() {
    let mut larger = hill();
    larger.start = [-1500.0, 0.0];
    larger.end = [1500.0, 0.0];
    larger.control = Some([0.0, 1050.0]);
    assert!(
      larger.subdivision_count() > hill().subdivision_count(),
      "a 10x curve should need more pieces"
    );
  }

  #[test]
  fn subdivision_count_is_clamped_at_both_ends() {
    let mut coarse = hill();
    coarse.subdivisions = Some(0);
    assert_eq!(coarse.subdivision_count(), MIN_SUBDIVISIONS);

    let mut absurd = hill();
    absurd.subdivisions = Some(100_000);
    assert_eq!(absurd.subdivision_count(), MAX_SUBDIVISIONS);

    // The derived count is clamped too, not just a declared one.
    let mut enormous = hill();
    enormous.control = Some([0.0, 10_000_000.0]);
    assert_eq!(enormous.subdivision_count(), MAX_SUBDIVISIONS);
  }

  #[test]
  fn declared_subdivisions_are_used_as_given() {
    let mut faceted = hill();
    faceted.subdivisions = Some(3);
    assert_eq!(faceted.subdivision_count(), 3);
    assert_eq!(faceted.sample_points().len(), 4);
  }

  #[test]
  fn quad_count_matches_subdivision_count() {
    let segment = hill();
    assert_eq!(
      segment.ribbon().quad_count(),
      segment.subdivision_count() as usize
    );
  }

  #[test]
  fn neighbouring_quads_share_their_edge_exactly() {
    let ribbon = hill().ribbon();
    for index in 1..ribbon.quad_count() {
      let [_, _, right_far, left_far] = ribbon.quad(index - 1);
      let [left_near, right_near, _, _] = ribbon.quad(index);
      // Bit-for-bit, not approximately: both quads read the same stored rail, so any drift here
      // would mean the ribbon can open a seam.
      assert_eq!(left_near, left_far);
      assert_eq!(right_near, right_far);
    }
  }

  #[test]
  fn ribbon_is_a_full_thickness_wide_at_its_ends() {
    let segment = hill();
    let ribbon = segment.ribbon();
    let [left_near, right_near, _, _] = ribbon.quad(0);
    assert!((left_near.distance(right_near) - segment.thickness).abs() < 0.001);

    let [_, _, right_far, left_far] = ribbon.quad(ribbon.quad_count() - 1);
    assert!((left_far.distance(right_far) - segment.thickness).abs() < 0.001);
  }

  #[test]
  fn every_generated_position_is_finite() {
    let ribbon = hill().ribbon();
    for index in 0..ribbon.quad_count() {
      for corner in ribbon.quad(index) {
        assert!(corner.is_finite(), "quad {index} has a non-finite corner");
      }
    }
    assert!(ribbon.centroid.is_finite());
  }

  #[test]
  fn centroid_sits_between_the_endpoints() {
    // Not the midpoint of the chord -- the samples bunch toward the peak -- but it must at least
    // land inside the curve's span, which is what makes it a useful entity origin.
    let ribbon = hill().ribbon();
    assert!(ribbon.centroid.x.abs() < 1.0);
    assert!(ribbon.centroid.y > 0.0 && ribbon.centroid.y < 52.5);
  }

  #[test]
  fn a_curve_bent_tighter_than_its_thickness_is_rejected() {
    let mut sharp = hill();
    // This hill is tightest at its apex, where B' is (300, 0) and B'' is (0, -420) -- a radius of
    // 300^3 / (4 * 31500), or ~214. A thickness of 500 needs 250, so it is rejected.
    sharp.thickness = 500.0;
    assert!(matches!(
      sharp.validate_curvature(),
      Err(SegmentError::TooSharpForThickness { .. })
    ));
    // The same curve at a workable thickness is fine, so the check is about the pairing rather
    // than about the curve alone.
    assert!(hill().validate_curvature().is_ok());
  }

  #[test]
  fn a_control_point_folding_the_curve_back_is_rejected() {
    let mut folded = straight();
    // Collinear and well outside the endpoints: the curve runs out past `control`, stops, and
    // comes back along itself.
    folded.control = Some([600.0, 0.0]);
    assert!(matches!(
      folded.validate_curvature(),
      Err(SegmentError::FoldedCurve)
    ));
  }

  #[test]
  fn a_non_finite_control_point_is_rejected() {
    let mut broken = hill();
    broken.control = Some([0.0, f32::NAN]);
    assert!(matches!(
      broken.validate(),
      Err(SegmentError::NonFiniteControl(_))
    ));
  }

  #[test]
  fn a_valid_curve_passes_full_validation() {
    assert!(hill().validate().is_ok());
    assert!(straight().validate().is_ok());
  }
}

