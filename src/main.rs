// Core Bevy imports used on every target: the input-condition helper and the prelude (App, Commands, etc.).
use avian2d::prelude::{Gravity, PhysicsPlugins, SubstepCount};
use bevy::prelude::*;

// Avian's default Gravity (-9.81) is tuned for meter-scale units; our world uses pixel-scale
// coordinates, so it needs to be much larger to produce a visible fall speed.
const GRAVITY: f32 = 400.0;

mod camera_follow;
mod components;
mod create_static_solid_surface;
mod levels;
mod player;
mod setup;

use camera_follow::camera_follow;
use levels::level1::level1;
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
    .insert_resource(SubstepCount(26))
    // Run `setup` once at startup to spawn the camera, shapes, and UI text.
    .add_systems(Startup, setup)
    .add_systems(Startup, level1);

  app.add_systems(Update, (player_input, camera_follow).chain());
  // Start the Bevy event loop; this blocks until the app exits.
  app.run();
}
