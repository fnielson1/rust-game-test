use crate::components::{MainCamera, Player};
use bevy::prelude::{Query, Res, Time, Transform, With, Without};
use bevy::window::{PrimaryWindow, Window};

// Higher = camera catches up to the player faster (less lag, less smoothing).
const CAMERA_SMOOTHING: f32 = 6.0;
// Where the player should sit vertically on screen: 0.5 = dead center, 1/3 = lower third.
const PLAYER_SCREEN_HEIGHT_FRACTION: f32 = 1.0 / 4.0;

pub fn camera_follow(
  time: Res<Time>,
  player_query: Query<&Transform, With<Player>>,
  mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
  window_query: Query<&Window, With<PrimaryWindow>>,
) {
  let Ok(player_transform) = player_query.single() else {
    return;
  };
  let Ok(mut camera_transform) = camera_query.single_mut() else {
    return;
  };
  let Ok(window) = window_query.single() else {
    return;
  };
  // Camera translation is the screen center, so to pin the player at
  // `PLAYER_SCREEN_HEIGHT_FRACTION` up from the bottom, shift the camera above the
  // player by however far that fraction sits from the vertical midpoint (0.5).
  let vertical_offset = (0.5 - PLAYER_SCREEN_HEIGHT_FRACTION) * window.height();
  let mut target = player_transform.translation;
  target.y += vertical_offset;
  target.z = camera_transform.translation.z;
  let t = (CAMERA_SMOOTHING * time.delta_secs()).min(1.0);
  camera_transform.translation = camera_transform.translation.lerp(target, t);
}
