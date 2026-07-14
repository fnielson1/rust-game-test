// Core Bevy imports used on every target: the input-condition helper and the prelude (App, Commands, etc.).
use bevy::{input::common_conditions::input_toggle_active, prelude::*};

// Marker for the triangle, which spins on the opposite schedule from everything else
// (it rotates exactly while the circle/rectangle are paused, and vice versa).
#[derive(Component)]
struct IdleSpinner;

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

  // Spawn the circle, then attach the rectangle to it as a child. Child transforms are
  // relative to the parent, so rotating/moving the circle carries the rectangle with it
  // — the two entities behave as a single rigid body.
  commands
    .spawn((
      Mesh2d(meshes.add(Circle::new(50.0))),
      MeshMaterial2d(materials.add(Color::hsl(0., 0.95, 0.7))),
      Transform::from_xyz(0.0, 0.0, 0.0),
    ))
    .with_child((
      Mesh2d(meshes.add(Rectangle::new(1.0, 50.0))),
      MeshMaterial2d(materials.add(Color::hsl(220., 0.95, 0.7))),
      Transform::from_xyz(0.0, 0.0, 0.1),
    ));

  commands.spawn((
    Mesh2d(meshes.add(Triangle2d::new(
      Vec2::new(20.0, 20.0),
      Vec2::new(10.0, 20.0),
      Vec2::new(10.0, 10.0),
    ))),
    MeshMaterial2d(materials.add(Color::hsl(100.0, 0.95, 0.7))),
    Transform::from_xyz(60.0, 0.0, 0.2),
    IdleSpinner,
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

// System that spins every root mesh entity around the Z axis each frame, unless paused via `R`.
// `Without<ChildOf>` excludes child entities (like the rectangle) so they aren't rotated a
// second time on top of the parent's rotation — they follow the parent via transform propagation.
fn rotate(
  mut query: Query<&mut Transform, (With<Mesh2d>, Without<ChildOf>, Without<IdleSpinner>)>,
  time: Res<Time>,
) {
  for mut transform in &mut query {
    // Rotate at a constant angular speed, scaled by elapsed frame time for frame-rate independence.
    transform.rotate_z(time.delta_secs() / 2.0);
  }
}

// Spins the triangle only while `rotate`/`counter_rotate_children` are paused (see the opposing
// `input_toggle_active` defaults in `main`).
fn rotate_idle_spinner(mut query: Query<&mut Transform, With<IdleSpinner>>, time: Res<Time>) {
  for mut transform in &mut query {
    transform.rotate_z(time.delta_secs() / 2.0);
  }
}

// Spins child mesh entities (like the rectangle) the opposite way, in world space, from their
// parent. A child's local rotation composes on top of the parent's, so to end up rotating at
// -speed in world space while the parent rotates at +speed, the child's local rotation must
// move at -2*speed (which cancels the parent's +speed and adds an equal -speed on top).
fn counter_rotate_children(
  mut query: Query<&mut Transform, (With<Mesh2d>, With<ChildOf>)>,
  time: Res<Time>,
) {
  for mut transform in &mut query {
    transform.rotate_z(-time.delta_secs());
  }
}
