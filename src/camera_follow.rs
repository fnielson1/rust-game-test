use crate::components::{Grounded, MainCamera, Player, SolidSurface};
use avian2d::prelude::{ColliderAabb, LinearVelocity};
use bevy::prelude::{Has, Local, Query, Res, Time, Transform, With, Without};
use bevy::window::{PrimaryWindow, Window};

// Higher = camera catches up to the player faster (less lag, less smoothing).
const CAMERA_SMOOTHING: f32 = 6.0;
// Lower than CAMERA_SMOOTHING so the on-screen anchor point eases toward its target
// instead of snapping, which would otherwise yank the lerp target around on every
// grounded/airborne transition.
const FRACTION_SMOOTHING: f32 = 2.5;
// Where the player should sit vertically on screen: 0.5 = dead center, 1/3 = lower third.
const GROUNDED_SCREEN_HEIGHT_FRACTION: f32 = 1.0 / 4.0;
const FALLING_SCREEN_HEIGHT_FRACTION: f32 = 3.0 / 4.0;
// Downward speed (world units/sec) at which the falling anchor is fully applied. Below
// this, the anchor eases proportionally to fall speed rather than snapping, so small
// vertical jitter while grounded (e.g. rolling over bumps) doesn't read as "falling".
const FALL_SPEED_FOR_FULL_TRANSITION: f32 = 100.0;
// Upward speed below which the anchor starts easing toward "falling", so the transition
// begins on the way up to the apex instead of waiting for velocity.y to cross zero.
const APEX_APPROACH_SPEED: f32 = 400.0;
// How many player heights of empty space below the lowest on-screen solid surface the
// camera is allowed to reveal, so a long fall doesn't pan the camera down into open void.
const MAX_HEIGHT_BELOW_GROUND: f32 = 100.0;

pub fn camera_follow(
  time: Res<Time>,
  mut screen_height_fraction: Local<Option<f32>>,
  player_query: Query<(&Transform, &LinearVelocity, Has<Grounded>), With<Player>>,
  mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
  solid_query: Query<&ColliderAabb, With<SolidSurface>>,
  window_query: Query<&Window, With<PrimaryWindow>>,
) {
  let Ok((player_transform, velocity, grounded)) = player_query.single() else {
    return;
  };
  let Ok(mut camera_transform) = camera_query.single_mut() else {
    return;
  };
  let Ok(window) = window_query.single() else {
    return;
  };
  // Falling players are shown higher on screen (more room to see what's below them);
  // grounded players sit lower (more room to see what's ahead/above). While grounded,
  // vertical velocity is just contact-resolution noise (e.g. rolling over bumps), so
  // it's ignored entirely; airborne, the anchor ramps from APEX_APPROACH_SPEED (still
  // rising, but slowing toward the apex) down through 0 to -FALL_SPEED_FOR_FULL_TRANSITION,
  // so it's already easing toward "falling" before the player actually starts falling.
  let fall_amount = if grounded {
    0.0
  } else {
    ((APEX_APPROACH_SPEED - velocity.y) / (APEX_APPROACH_SPEED + FALL_SPEED_FOR_FULL_TRANSITION))
      .clamp(0.0, 1.0)
  };
  let target_fraction = GROUNDED_SCREEN_HEIGHT_FRACTION
    + (FALLING_SCREEN_HEIGHT_FRACTION - GROUNDED_SCREEN_HEIGHT_FRACTION) * fall_amount;
  let current_fraction = screen_height_fraction.get_or_insert(target_fraction);
  let fraction_t = (FRACTION_SMOOTHING * time.delta_secs()).min(1.0);
  *current_fraction += (target_fraction - *current_fraction) * fraction_t;
  // Camera translation is the screen center, so to pin the player at
  // `current_fraction` up from the bottom, shift the camera above the
  // player by however far that fraction sits from the vertical midpoint (0.5).
  let vertical_offset = (0.5 - *current_fraction) * window.height();
  let mut target = player_transform.translation;
  target.y += vertical_offset;
  target.z = camera_transform.translation.z;

  // Clamp against the void: never let the camera reveal more than
  // MAX_PLAYER_HEIGHTS_BELOW_GROUND worth of empty space beneath the lowest solid
  // surface currently spanning the screen's width, no matter how fast the player falls.
  // Applied to the target (not the final translation) so the lerp below eases the
  // camera into and out of the clamp instead of snapping onto it.
  let screen_min_x = target.x - window.width() / 2.0;
  let screen_max_x = target.x + window.width() / 2.0;
  let lowest_on_screen_solid_bottom = solid_query
    .iter()
    .filter(|aabb| aabb.max.x >= screen_min_x && aabb.min.x <= screen_max_x)
    .map(|aabb| aabb.min.y)
    .fold(f32::INFINITY, f32::min);
  if lowest_on_screen_solid_bottom.is_finite() {
    let min_camera_y =
      lowest_on_screen_solid_bottom - MAX_HEIGHT_BELOW_GROUND + window.height() / 2.0;
    target.y = target.y.max(min_camera_y);
  }

  let t = (CAMERA_SMOOTHING * time.delta_secs()).min(1.0);
  camera_transform.translation = camera_transform.translation.lerp(target, t);
}
