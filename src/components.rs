use bevy::prelude::*;

/// Marker for the triangle, which spins on the opposite schedule from everything else
/// (it rotates exactly while the circle/rectangle are paused, and vice versa).
#[derive(Component)]
pub struct IdleSpinner;

#[derive(Component)]
pub struct SolidSurface;

#[derive(Component)]
pub struct Player;
