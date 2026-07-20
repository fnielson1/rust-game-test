use bevy::prelude::Component;

#[derive(Component)]
pub struct SolidSurface;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MainCamera;

/// Marker inserted on the `Player` while a downward shapecast reports ground contact,
/// and removed otherwise. `SparseSet` storage since it's toggled every frame rather than
/// queried/iterated in bulk, which is cheaper for components that come and go often.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;
