use crate::components::PlayerCamera;
use crate::player::setup_player::setup_player;
use crate::world_bounds::{WORLD_HEIGHT, WORLD_WIDTH};
use bevy::camera::ScalingMode;
use bevy::prelude::{
  Assets, Camera2d, ColorMaterial, Commands, Mesh, OrthographicProjection, Projection, ResMut,
};

/// Startup system: spawns the camera, a row of solid 2D shapes, a row of their ring/outline
/// counterparts, and an on-screen instructions text.
pub fn setup(
  mut commands: Commands,
  // Mesh storage; `meshes.add(...)` uploads geometry and returns a handle to it.
  meshes: ResMut<Assets<Mesh>>,
  // Material storage; `materials.add(color)` creates a solid-color material and returns a handle.
  materials: ResMut<Assets<ColorMaterial>>,
) {
  // Spawn a 2D camera so anything rendered below is actually visible. AutoMin scales the
  // WORLD_WIDTH x WORLD_HEIGHT reference area to fill the window, so the whole scene zooms
  // with the window (bigger on fullscreen) instead of a fixed 1:1 pixel mapping that would
  // just reveal more or less of the world.
  commands.spawn((
    Camera2d,
    PlayerCamera,
    Projection::Orthographic(OrthographicProjection {
      scaling_mode: ScalingMode::AutoMin {
        min_width: WORLD_WIDTH,
        min_height: WORLD_HEIGHT,
      },
      ..OrthographicProjection::default_2d()
    }),
  ));
  // Player
  setup_player(commands, meshes, materials);
}
