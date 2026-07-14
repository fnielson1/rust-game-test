use crate::components::Player;
use bevy::prelude::*;

/// Startup system: spawns the camera, a row of solid 2D shapes, a row of their ring/outline
/// counterparts, and an on-screen instructions text.
pub fn setup(
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
      Player,
    ))
    .with_child((
      Mesh2d(meshes.add(Rectangle::new(1.0, 50.0))),
      MeshMaterial2d(materials.add(Color::hsl(220., 0.95, 0.7))),
      Transform::from_xyz(0.0, 0.0, 0.1),
    ));
}
