use avian2d::prelude::AngularVelocity;
use bevy::prelude::{Query, With};

use crate::components::Player;

const ROTATION_SPEED_DIVISOR: f32 = 1.0;
// Radians/sec the Player spins at while active; matches the old `1.0 / ROTATION_SPEED_DIVISOR`
// per-second rate from directly mutating Transform.
const ROTATION_SPEED: f32 = 1.0 / ROTATION_SPEED_DIVISOR;

/// Sets the Player's angular velocity so avian2d's physics step integrates the spin into
/// `Transform` itself. Player is a dynamic `RigidBody`, so writing `Transform` directly here
/// (as this used to) would fight the physics engine, which also writes `Transform` each step.
pub fn rotate(mut query: Query<&mut AngularVelocity, With<Player>>) {
  for mut angular_velocity in &mut query {
    angular_velocity.0 = ROTATION_SPEED;
  }
}

/// Zeroes the Player's angular velocity while rotation is paused. Needed because `rotate` only
/// runs while the `R` toggle is active — without this, angular velocity would keep whatever
/// value `rotate` last set and physics would spin the player forever once toggled off.
pub fn stop_rotation(mut query: Query<&mut AngularVelocity, With<Player>>) {
  for mut angular_velocity in &mut query {
    angular_velocity.0 = 0.0;
  }
}
