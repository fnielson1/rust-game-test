use bevy::prelude::Component;

#[derive(Component)]
pub struct SolidSurface;

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
