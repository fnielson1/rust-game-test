//! Turns a loaded [`LevelAsset`] into collidable geometry, and rebuilds it when it changes.
//!
//! Spawning is deliberately split from loading. [`crate::levels::level1`] only says *which* level
//! to load and asks for a rebuild; everything that actually creates entities lives in
//! [`spawn_level`]. That split is what lets three different triggers share one implementation:
//!
//! - startup, once the asset finishes loading (`AssetEvent::LoadedWithDependencies`)
//! - the file being saved while the game runs (`AssetEvent::Modified`)
//! - a code hot patch, which re-runs `level1` after `despawn_world` has cleared the world
//!
//! The third is the reason [`LevelRespawn`] exists rather than reacting to asset events alone.
//! On a hot patch the handle is already loaded, so no asset event fires -- without an explicit
//! request the level would be despawned and never rebuilt.

use crate::components::LevelSegment;
use crate::create_static_solid::create_static_solid;
use crate::levels::level_asset::LevelAsset;
use avian2d::prelude::Collider;
use bevy::asset::AssetLoadFailedEvent;
use bevy::prelude::{
  Assets, ColorMaterial, Commands, Entity, Handle, Mesh, MessageReader, Query, Rectangle, Res,
  ResMut, Resource, With, error, info, warn,
};
use bevy::{asset::AssetEvent, color::Color};

/// Depth for level geometry, matching what the hardcoded level used so the player still draws
/// in front of it.
const SEGMENT_Z: f32 = 0.1;
const SEGMENT_COLOR_HUE: f32 = 100.0;

/// The level file currently loaded. Absent until [`crate::levels::level1`] has run once.
#[derive(Resource)]
pub struct LevelHandle(pub Handle<LevelAsset>);

/// Set when the level should be rebuilt from [`LevelHandle`] on the next [`spawn_level`] run.
///
/// A flag rather than a message because the request is idempotent -- three triggers firing in
/// one frame should produce one rebuild, not three.
#[derive(Resource, Default)]
pub struct LevelRespawn(pub bool);

impl LevelRespawn {
  pub fn request(&mut self) {
    self.0 = true;
  }
}

/// Rebuilds the level's geometry when a rebuild has been requested.
///
/// Clears the previous segments first, so repeated rebuilds replace rather than accumulate.
pub fn spawn_level(
  mut commands: Commands,
  mut respawn: ResMut<LevelRespawn>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
  // Optional: `level1` may not have run yet on the very first frame.
  level_handle: Option<Res<LevelHandle>>,
  levels: Res<Assets<LevelAsset>>,
  existing: Query<Entity, With<LevelSegment>>,
) {
  if !respawn.0 {
    return;
  }
  // Cleared unconditionally: if the asset isn't ready yet, the load event will request again.
  // Leaving it set would retry this every frame until the file arrives.
  respawn.0 = false;

  let Some(handle) = level_handle else {
    return;
  };

  let mut despawned = 0_usize;
  for entity in &existing {
    commands.entity(entity).despawn();
    despawned += 1;
  }

  let Some(level) = levels.get(&handle.0) else {
    return;
  };

  let mut spawned = 0_usize;
  for (index, segment) in level.segments.iter().enumerate() {
    if let Err(reason) = segment.validate() {
      warn!("level segment {index} skipped: {reason}");
      continue;
    }
    let color = Color::hsl(SEGMENT_COLOR_HUE, 0.95, 0.7);
    // Both arms return the same tuple from `create_static_solid`, so a curve and a straight
    // segment spawn as one entity apiece, indistinguishable to everything downstream -- the
    // despawn count above, the `HotReloadable` marker, and the grounding check all stay as they
    // were. Only the mesh, transform, and collider handed in differ.
    let solid = if segment.is_curved() {
      let ribbon = segment.ribbon();
      // The only curve failure `validate` can't catch: a quad too degenerate for avian to hull.
      // Skipped and reported like any other authoring mistake rather than panicking.
      let Some(collider) = ribbon.collider() else {
        warn!("level segment {index} skipped: curve produced no collidable surface");
        continue;
      };
      create_static_solid(
        &mut meshes,
        &mut materials,
        ribbon.mesh(),
        color,
        // Translation only, and vertices already in local space around the centroid -- a curve
        // has no single angle for a rotation to carry.
        ribbon.transform(SEGMENT_Z),
        collider,
      )
    } else {
      let length = segment.length();
      create_static_solid(
        &mut meshes,
        &mut materials,
        Rectangle::new(length, segment.thickness),
        color,
        segment.transform(SEGMENT_Z),
        // Avian rotates this by the transform above, so one axis-aligned rectangle collider
        // covers every angle a segment can be drawn at.
        Collider::rectangle(length, segment.thickness),
      )
    };
    spawned += 1;
    commands.spawn((solid, LevelSegment));
  }

  // Both counts, not just the new one: a rebuild that despawns fewer than it spawned is how
  // duplicate geometry would accumulate, and stacked identical segments are invisible on screen.
  info!("level rebuilt: despawned {despawned}, spawned {spawned}");
}

/// Requests a rebuild when the level asset finishes loading or is changed on disk.
///
/// `Modified` is what makes saving a level file rebuild it live; it only ever fires in builds
/// carrying bevy's `file_watcher` feature, which the `hotpatch` feature pulls in.
pub fn respawn_on_level_asset_change(
  mut asset_events: MessageReader<AssetEvent<LevelAsset>>,
  mut respawn: ResMut<LevelRespawn>,
  level_handle: Option<Res<LevelHandle>>,
) {
  let Some(handle) = level_handle else {
    return;
  };
  for event in asset_events.read() {
    let is_current_level =
      event.is_loaded_with_dependencies(&handle.0) || event.is_modified(&handle.0);
    if is_current_level {
      respawn.request();
    }
  }
}

/// Reports a level file that couldn't be found or parsed, naming the path that was tried.
///
/// Bevy's own load-failure logging is easy to lose in startup noise, and the path is the single
/// most useful thing to see here: a native run launched without `BEVY_ASSET_ROOT` pointing at the
/// repo root fails exactly this way.
pub fn report_level_load_failures(mut failures: MessageReader<AssetLoadFailedEvent<LevelAsset>>) {
  for failure in failures.read() {
    error!("failed to load level `{}`: {}", failure.path, failure.error);
  }
}
