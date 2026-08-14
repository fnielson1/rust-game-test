// Core Bevy imports used on every target: the input-condition helper and the prelude (App, Commands, etc.).
use avian2d::prelude::{Gravity, PhysicsPlugins, SubstepCount};
use bevy::prelude::*;

// Avian's default Gravity (-9.81) is tuned for meter-scale units; our world uses pixel-scale
// coordinates, so it needs to be much larger to produce a visible fall speed.
const GRAVITY: f32 = 400.0;

mod app_state;
mod camera_follow;
mod components;
mod create_static_solid;
mod input_config;
mod levels;
mod menu;
mod player;
mod setup;
mod world_bounds;

use app_state::{AppState, toggle_menu};
use camera_follow::camera_follow;
use input_config::{KeyBindings, RebindError, RebindRequest, rebind_capture};
use levels::level1::level1;
use menu::{
  cancel_rebind_on_outside_click, clear_rebind_state, handle_backdrop_click, handle_cog_click,
  handle_row_clicks, spawn_cog_button, spawn_menu, update_binding_rows, update_error_label,
};
use player::grounded::update_grounded;
use player::player_input::player_input;
use setup::setup;

fn main() {
  // Create the Bevy App that will own all plugins, systems, and resources.
  let mut app = App::new();
  app
    .add_plugins((
      // Bevy's standard plugin bundle: windowing, rendering, input, asset loading, etc.
      DefaultPlugins,
      // Avian's 2D physics: rigid bodies, colliders, gravity, and collision resolution.
      PhysicsPlugins::default(),
    ))
    .insert_resource(Gravity(Vec2::NEG_Y * GRAVITY))
    // More substeps = the friction solver resolves the spin-to-roll grip more gradually
    // instead of correcting the whole slip velocity in one step (avian2d default is 6).
    .insert_resource(SubstepCount(6))
    .init_state::<AppState>()
    .init_resource::<KeyBindings>()
    .init_resource::<RebindRequest>()
    .init_resource::<RebindError>()
    // Run `setup` once at startup to spawn the camera, shapes, and UI text.
    .add_systems(Startup, setup)
    .add_systems(Startup, level1)
    .add_systems(Startup, spawn_cog_button)
    .add_systems(OnEnter(AppState::Menu), spawn_menu)
    .add_systems(OnExit(AppState::Menu), clear_rebind_state);

  app.add_systems(
    Update,
    (
      update_grounded,
      player_input.run_if(in_state(AppState::InGame)),
      camera_follow,
    )
      .chain(),
  );
  app.add_systems(Update, (toggle_menu, handle_cog_click));
  app.add_systems(
    Update,
    (
      rebind_capture,
      // Must clear the old request before the row handler records the newly clicked one.
      cancel_rebind_on_outside_click.before(handle_row_clicks),
      handle_row_clicks,
      handle_backdrop_click,
      update_binding_rows,
      update_error_label,
    )
      .run_if(in_state(AppState::Menu)),
  );
  // Start the Bevy event loop; this blocks until the app exits.
  app.run();
}
