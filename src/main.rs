// Core Bevy imports used on every target: the input-condition helper and the prelude (App, Commands, etc.).
use bevy::{input::common_conditions::input_toggle_active, prelude::*};

mod components;
mod create_solid_surface;
mod levels;
mod rotation;
mod setup;

use levels::level1::level1;
use rotation::{counter_rotate_children, rotate, rotate_idle_spinner};
use setup::setup;

fn main() {
  // Create the Bevy App that will own all plugins, systems, and resources.
  let mut app = App::new();
  app
    .add_plugins((
      // Bevy's standard plugin bundle: windowing, rendering, input, asset loading, etc.
      DefaultPlugins,
    ))
    // Run `setup` once at startup to spawn the camera, shapes, and UI text.
    .add_systems(Startup, setup)
    .add_systems(Startup, level1);
  // Run `rotate` every frame, but only while rotation hasn't been toggled off by pressing R
  // (input_toggle_active(false, ...) starts in the "active" state and flips each press of R).
  app.add_systems(
    Update,
    (rotate, counter_rotate_children).run_if(input_toggle_active(false, KeyCode::KeyR)),
  );
  // The triangle does the opposite: it only spins while the others are paused. Giving this
  // tracker the opposite starting default (true vs. false above) means it toggles on the same
  // R presses but is always the inverse of the condition above.
  app.add_systems(
    Update,
    rotate_idle_spinner.run_if(input_toggle_active(true, KeyCode::KeyR)),
  );
  // Start the Bevy event loop; this blocks until the app exits.
  app.run();
}
