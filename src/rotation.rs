use bevy::prelude::*;

use crate::components::Player;

const ROTATION_SPEED_DIVISOR: f32 = 1.0;

/// System that spins every root mesh entity around the Z axis each frame, unless paused via `R`.
/// `Without<ChildOf>` excludes child entities (like the rectangle) so they aren't rotated a
/// second time on top of the parent's rotation — they follow the parent via transform propagation.
pub fn rotate(mut query: Query<&mut Transform, With<Player>>, time: Res<Time>) {
  for mut transform in &mut query {
    // Rotate at a constant angular speed, scaled by elapsed frame time for frame-rate independence.
    transform.rotate_z(time.delta_secs() / ROTATION_SPEED_DIVISOR);
  }
}

/// Spins child mesh entities (like the rectangle) the opposite way, in world space, from their
/// parent. A child's local rotation composes on top of the parent's, so to end up rotating at
/// -speed in world space while the parent rotates at +speed, the child's local rotation must
/// move at -2*speed (which cancels the parent's +speed and adds an equal -speed on top). This is
/// derived from `ROTATION_SPEED_DIVISOR` (the same constant `rotate` uses) so the two systems
/// can't drift out of sync if the rotation speed changes.
pub fn counter_rotate_children(
  mut query: Query<&mut Transform, (With<Mesh2d>, With<ChildOf>)>,
  time: Res<Time>,
) {
  for mut transform in &mut query {
    transform.rotate_z(-2.0 * time.delta_secs() / ROTATION_SPEED_DIVISOR);
  }
}
