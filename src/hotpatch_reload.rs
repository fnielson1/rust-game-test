//! Re-runs world construction when a hot patch lands.
//!
//! Subsecond swaps function *bodies*, so a patch only shows up in code that runs again.
//! `Startup` systems have already run by then: editing `level1` or `setup_player` updates the
//! function, but nothing calls it a second time, and the entities they spawned keep the values
//! they were built with. That's why a changed `FLOOR_HEIGHT` or `PLAYER_RADIUS` would otherwise
//! only appear after a restart.
//!
//! Bevy fires a `HotPatched` message on every patch, so [`HotPatchReloadPlugin`] schedules
//! `despawn_world` followed by the spawn systems behind that message, rebuilding them from the
//! patched code.
//!
//! Level geometry is a special case since it moved into JSON. `level1` only requests a rebuild;
//! `spawn_level` does the spawning. The request is what makes this work at all -- re-running
//! `level1` reloads an already-loaded asset, which fires no asset event, so without the explicit
//! request `despawn_world` would clear the level and nothing would bring it back.
//!
//! The spawn systems have to be *scheduled* rather than invoked through
//! `Commands::run_system_cached`. Bevy only calls `System::refresh_hotpatch` on systems an
//! executor owns, so a cached system keeps running whichever body it captured on its first
//! call and silently ignores every later patch.
//!
//! Only compiled under the `hotpatch` feature -- `HotPatched` itself doesn't exist without it,
//! and release builds have no reason to carry a world rebuilder.

use crate::components::HotReloadable;
use crate::levels::level1::level1;
use crate::levels::spawn_level::spawn_level;
use crate::player::setup_player::setup_player;
use bevy::ecs::HotPatched;
use bevy::ecs::schedule::common_conditions::on_message;
use bevy::prelude::{App, Commands, Entity, IntoScheduleConfigs, Plugin, Query, Update, With};

/// Rebuilds the spawned world on every hot patch, so edits to spawn code show up without a
/// restart.
pub struct HotPatchReloadPlugin;

impl Plugin for HotPatchReloadPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      // ---- Add new spawn systems here (level2, level3, ...) ----
      // Order matters only in that `despawn_world` comes first; `chain` gives it a sync point
      // to apply through, so the old world is gone before the new one spawns. Whatever a
      // system listed here spawns must carry `HotReloadable`, or the rebuild stacks a
      // duplicate on top of the old copy every patch.
      // `level1` no longer spawns anything itself -- it requests a level rebuild, which
      // `spawn_level` performs. Ordering before `spawn_level` keeps that rebuild in this same
      // frame, so a patch never leaves a frame with the world despawned and not yet rebuilt.
      (despawn_world, setup_player, level1)
        .chain()
        .before(spawn_level)
        .run_if(on_message::<HotPatched>),
    );
  }
}

/// Despawns everything marked [`HotReloadable`] so the spawn systems above can rebuild it.
///
/// The marker is what keeps this in sync with the list above: an entity carries it if and only
/// if one of those systems will spawn it again. Marking something nothing respawns deletes it
/// for the session (the camera and cog button are deliberately unmarked); respawning something
/// unmarked stacks a duplicate per patch.
///
/// Children don't need the marker -- `despawn` recurses into `Children`, so the player's child
/// rectangle goes with it rather than leaking a copy per rebuild.
///
/// Note this resets the player to its spawn point on every patch, including patches to
/// unrelated files. That's the cost of seeing `setup_player` edits take effect; drop
/// `HotReloadable` from the player's spawn if you'd rather keep your position and give up live
/// player edits.
fn despawn_world(mut commands: Commands, entities: Query<Entity, With<HotReloadable>>) {
  for entity in &entities {
    commands.entity(entity).despawn();
  }
}
