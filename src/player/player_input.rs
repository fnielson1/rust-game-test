use crate::components::{CoyoteTimer, Grounded, Player};
use crate::input_config::{InputAction, KeyBindings};
use avian2d::prelude::{AngularVelocity, Collider, LinearVelocity};
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
// While airborne there's no ground contact, so friction can't turn spin into rolling
// motion — spinning the ball midair wouldn't move it. Air control instead nudges
// horizontal velocity directly, at a slower rate than the near-instant grounded response
// so it feels like drift rather than full ground handling.
const AIR_CONTROL_ACCEL: f32 = 200.0;
// Air drift caps at this fraction of the ball's rolling top speed, so air control can't
// outrun (and stays a bit weaker than) what's reachable by rolling on the ground.
// const AIR_SPEED_FRACTION: f32 = 1.0;

type PlayerQueryData<'a> = (
  &'a mut LinearVelocity,
  &'a mut AngularVelocity,
  &'a mut CoyoteTimer,
  Option<&'a Grounded>,
  &'a Collider,
);

pub fn player_input(
  keys: Res<ButtonInput<KeyCode>>,
  bindings: Res<KeyBindings>,
  time: Res<Time>,
  mut query: Query<PlayerQueryData, With<Player>>,
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
    for (mut linear_velocity, mut angular_velocity, _, grounded, collider) in &mut query {
      // Reversing spin gets a stronger kick than accelerating further the same way.
      // Applies whether grounded or airborne, so the ball keeps spinning normally in the air.
      let multiplier = if angular_velocity.0 * input_sign < 0.0 {
        ROTATION_REVERSE_MULTIPLIER
      } else {
        1.0
      };
      let delta = input_sign * ROTATION_ACCEL * multiplier * time.delta_secs();
      angular_velocity.0 =
        (angular_velocity.0 + delta).clamp(-MAX_ROTATION_SPEED, MAX_ROTATION_SPEED);

      if grounded.is_none() {
        // Spinning alone doesn't move the ball while airborne (no ground contact for
        // friction to convert it into rolling motion), so also drift horizontally
        // directly. Rolling relation is velocity_x = -angular_velocity * radius, so a
        // positive (rotate_left) input needs to push velocity negative to drift the
        // same direction spin would otherwise roll on the ground.
        //
        // Read the radius straight from the entity's own collider shape rather than a
        // shared constant, so this stays correct even if the player's collider ever changes.
        let radius = collider
          .shape()
          .as_ball()
          .map(|ball| ball.radius)
          .unwrap_or(0.0);
        let max_player_speed = radius * MAX_ROTATION_SPEED;
        let max_air_speed = max_player_speed /* * AIR_SPEED_FRACTION*/;
        let air_delta = -input_sign * AIR_CONTROL_ACCEL * time.delta_secs();
        linear_velocity.0.x =
          (linear_velocity.0.x + air_delta).clamp(-max_air_speed, max_air_speed);
      }
    }
  }
  // Jump
  if keys.pressed(jump_key) {
    for (mut velocity, _, mut coyote_timer, _, _) in &mut query {
      if coyote_timer.0 <= COYOTE_TIME {
        velocity.0 = Vec2::new(velocity.0.x, JUMP_SPEED);
        // Push the timer past the window so this jump can't be re-triggered on the next
        // frame(s) while the key is still held and the player hasn't touched down again.
        coyote_timer.0 = f32::MAX;
      }
    }
  }
}
