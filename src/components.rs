use bevy::prelude::Component;

#[derive(Component)]
pub struct SolidSurface;

/// Marks an entity as owned by a hot-reloadable spawn system, so the hot-patch rebuild knows
/// to despawn it (see `hotpatch_reload`).
///
/// The rule this encodes: an entity carries this marker if and only if one of the systems the
/// rebuild re-runs will spawn it again. Marking something nothing respawns deletes it for the
/// rest of the session; respawning something unmarked stacks a duplicate on every patch.
///
/// Only root entities need it -- `despawn` recurses into `Children`.
///
/// Lives here rather than in `hotpatch_reload` so bundles can carry it unconditionally; it's
/// an empty marker, so it costs nothing in builds without the `hotpatch` feature.
#[derive(Component)]
pub struct HotReloadable;

/// Marks an entity spawned from a line segment in a level file.
///
/// Distinct from [`HotReloadable`] because the two sets differ: the player is hot-reloadable but
/// is not level geometry, so a level rebuild must not despawn it. `spawn_level` clears exactly
/// the entities carrying this marker before spawning the new set.
#[derive(Component)]
pub struct LevelSegment;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

/// Marker inserted on the `Player` while a downward shapecast reports ground contact,
/// and removed otherwise. `SparseSet` storage since it's toggled every frame rather than
/// queried/iterated in bulk, which is cheaper for components that come and go often.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// Seconds since the `Player` was last `Grounded`; reset to `0.0` on contact. Backs "coyote
/// time" — a short post-liftoff grace period where a jump still lands. Games can't require
/// pixel-perfect ground timing from a human, so this widens the input window without letting
/// the player float indefinitely: `player_input` only honors the jump while this is small.
#[derive(Component)]
pub struct CoyoteTimer(pub f32);
