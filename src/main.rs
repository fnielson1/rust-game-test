// Core Bevy imports used on every target: the input-condition helper and the prelude (App, Commands, etc.).
use bevy::{input::common_conditions::input_toggle_active, prelude::*};

fn main() {
  // Create the Bevy App that will own all plugins, systems, and resources.
  let mut app = App::new();
  app
    .add_plugins((
      // Bevy's standard plugin bundle: windowing, rendering, input, asset loading, etc.
      DefaultPlugins,
    ))
    // Run `setup` once at startup to spawn the camera, shapes, and UI text.
    .add_systems(Startup, setup);
  // Run `rotate` every frame, but only while rotation hasn't been toggled off by pressing R
  // (input_toggle_active(false, ...) starts in the "active" state and flips each press of R).
  app.add_systems(
    Update,
    rotate.run_if(input_toggle_active(false, KeyCode::KeyR)),
  );
  // Start the Bevy event loop; this blocks until the app exits.
  app.run();
}

// Startup system: spawns the camera, a row of solid 2D shapes, a row of their ring/outline
// counterparts, and an on-screen instructions text.
fn setup(
  mut commands: Commands,
  // Mesh storage; `meshes.add(...)` uploads geometry and returns a handle to it.
  mut meshes: ResMut<Assets<Mesh>>,
  // Material storage; `materials.add(color)` creates a solid-color material and returns a handle.
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  // Spawn a 2D camera so anything rendered below is actually visible.
  commands.spawn(Camera2d);

  // Spawn a single circle at the center of the screen.
  commands.spawn((
    Mesh2d(meshes.add(Circle::new(50.0))),
    MeshMaterial2d(materials.add(Color::hsl(0., 0.95, 0.7))),
    Transform::from_xyz(0.0, 0.0, 0.0),
  ));

  // Base instructions text shown on every target.
  let text = "Press 'R' to pause/resume rotation";

  // Spawn a UI text node pinned to the top-left corner of the screen.
  commands.spawn((
    Text::new(text),
    Node {
      // Absolute positioning relative to the UI root, not the layout flow.
      position_type: PositionType::Absolute,
      top: px(12),
      left: px(12),
      ..default()
    },
  ));
}

// System that spins every mesh entity around the Z axis each frame, unless paused via `R`.
fn rotate(mut query: Query<&mut Transform, With<Mesh2d>>, time: Res<Time>) {
  for mut transform in &mut query {
    // Rotate at a constant angular speed, scaled by elapsed frame time for frame-rate independence.
    transform.rotate_z(time.delta_secs() / 2.0);
  }
}
