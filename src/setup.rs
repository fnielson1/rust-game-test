use crate::player::setup_player::setup_player;
use bevy::prelude::{Assets, Camera2d, ColorMaterial, Commands, Mesh, ResMut};

/// Startup system: spawns the camera, a row of solid 2D shapes, a row of their ring/outline
/// counterparts, and an on-screen instructions text.
pub fn setup(
  mut commands: Commands,
  // Mesh storage; `meshes.add(...)` uploads geometry and returns a handle to it.
  meshes: ResMut<Assets<Mesh>>,
  // Material storage; `materials.add(color)` creates a solid-color material and returns a handle.
  materials: ResMut<Assets<ColorMaterial>>,
) {
  // Spawn a 2D camera so anything rendered below is actually visible.
  commands.spawn(Camera2d);
  // Player
  setup_player(commands, meshes, materials);
}
