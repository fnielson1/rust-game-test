use crate::components::Player;
use avian2d::prelude::{AngularVelocity, LinearVelocity};
use bevy::prelude::{ButtonInput, KeyCode, Query, Res, Time, Vec2, With};

// Radians/sec^2 the Player's spin accelerates by while the key is held.
const ROTATION_ACCEL: f32 = 40.0;
// Extra multiplier applied when the input opposes the current spin, so
// reversing direction feels snappier than accelerating further the same way.
const ROTATION_REVERSE_MULTIPLIER: f32 = 5.0;
const MAX_ROTATION_SPEED: f32 = 20.0;
const JUMP_SPEED: f32 = 500.0;

pub fn player_input(
  keys: Res<ButtonInput<KeyCode>>,
  time: Res<Time>,
  mut query: Query<(&mut LinearVelocity, &mut AngularVelocity), With<Player>>,
) {
  // Spacebar
  if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::ArrowUp) {
    for (mut velocity, _) in &mut query {
      velocity.0 = Vec2::new(velocity.0.x, JUMP_SPEED);
    }
  }
  // Left/Right
  if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::ArrowLeft) {
    let input_sign = if keys.pressed(KeyCode::ArrowLeft) {
      1.0
    } else {
      -1.0
    };
    for (_, mut angular_velocity) in &mut query {
      // Reversing spin gets a stronger kick than accelerating further the same way.
      let multiplier = if angular_velocity.0 * input_sign < 0.0 {
        ROTATION_REVERSE_MULTIPLIER
      } else {
        1.0
      };
      let delta = input_sign * ROTATION_ACCEL * multiplier * time.delta_secs();
      angular_velocity.0 =
        (angular_velocity.0 + delta).clamp(-MAX_ROTATION_SPEED, MAX_ROTATION_SPEED);
    }
  }
}
