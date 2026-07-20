use crate::components::{CoyoteTimer, Player};
use crate::input::{InputAction, KeyBindings};
use avian2d::prelude::{AngularVelocity, LinearVelocity};
use bevy::prelude::{ButtonInput, KeyCode, Query, Res, Time, Vec2, With};

// Radians/sec^2 the Player's spin accelerates by while the key is held.
const ROTATION_ACCEL: f32 = 40.0;
// Extra multiplier applied when the input opposes the current spin, so
// reversing direction feels snappier than accelerating further the same way.
const ROTATION_REVERSE_MULTIPLIER: f32 = 5.0;
const MAX_ROTATION_SPEED: f32 = 20.0;
const JUMP_SPEED: f32 = 500.0;
// "Coyote time": how long after leaving the ground a jump still counts. Human reaction
// time and one-frame-late input make exact-frame ground timing feel unresponsive, so this
// widens the window without letting the player jump arbitrarily late in the air.
const COYOTE_TIME: f32 = 0.15;

pub fn player_input(
  keys: Res<ButtonInput<KeyCode>>,
  bindings: Res<KeyBindings>,
  time: Res<Time>,
  mut query: Query<(&mut LinearVelocity, &mut AngularVelocity, &mut CoyoteTimer), With<Player>>,
) {
  let jump_key = bindings.bound_key(InputAction::Jump);
  let rotate_left_key = bindings.bound_key(InputAction::RotateLeft);
  let rotate_right_key = bindings.bound_key(InputAction::RotateRight);

  // Left/Right
  if keys.pressed(rotate_left_key) || keys.pressed(rotate_right_key) {
    let input_sign = if keys.pressed(rotate_left_key) {
      1.0
    } else {
      -1.0
    };
    for (_, mut angular_velocity, _) in &mut query {
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
  // Jump
  if keys.pressed(jump_key) {
    for (mut velocity, _, mut coyote_timer) in &mut query {
      if coyote_timer.0 <= COYOTE_TIME {
        velocity.0 = Vec2::new(velocity.0.x, JUMP_SPEED);
        // Push the timer past the window so this jump can't be re-triggered on the next
        // frame(s) while the key is still held and the player hasn't touched down again.
        coyote_timer.0 = f32::MAX;
      }
    }
  }
}
