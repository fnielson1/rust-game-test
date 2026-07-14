// Core Bevy imports used on every target: the input-condition helper and the prelude (App, Commands, etc.).
use avian2d::prelude::{Gravity, PhysicsPlugins};
use bevy::{input::common_conditions::input_toggle_active, prelude::*};

// Avian's default Gravity (-9.81) is tuned for meter-scale units; our world uses pixel-scale
// coordinates, so it needs to be much larger to produce a visible fall speed.
const GRAVITY: f32 = 400.0;

mod components;
mod create_solid_surface;
mod levels;
mod rotation;
mod setup;

use levels::level1::level1;
use rotation::{counter_rotate_children, rotate};
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
    // Run `setup` once at startup to spawn the camera, shapes, and UI text.
    .add_systems(Startup, setup)
    .add_systems(Startup, level1);
  // Run `rotate` every frame, but only while rotation hasn't been toggled off by pressing R
  // (input_toggle_active(false, ...) starts in the "active" state and flips each press of R).
  app.add_systems(
    Update,
    (rotate, counter_rotate_children).run_if(input_toggle_active(false, KeyCode::KeyR)),
  );
  // Start the Bevy event loop; this blocks until the app exits.
  app.run();
}
