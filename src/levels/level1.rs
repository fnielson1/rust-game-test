use crate::create_static_solid::create_static_solid;
use crate::world_bounds::{WORLD_HEIGHT, WORLD_WIDTH};
use avian2d::prelude::Collider;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::mesh::Mesh;
use bevy::prelude::{ColorMaterial, Commands, Rectangle, ResMut, Vec3};

const FLOOR_HEIGHT: f32 = 20.0;
const WALL_WIDTH: f32 = 20.0;

pub fn level1(
  mut commands: Commands,
  // Mesh storage; `meshes.add(...)` uploads geometry and returns a handle to it.
  mut meshes: ResMut<Assets<Mesh>>,
  // Material storage; `materials.add(color)` creates a solid-color material and returns a handle.
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  // The level is built from the fixed WORLD_WIDTH/WORLD_HEIGHT reference resolution, not the
  // live window size, so its layout stays consistent no matter what size window it's viewed
  // through (the camera's AutoMin scaling handles fitting it to the actual window).
  let width = WORLD_WIDTH;
  let height = WORLD_HEIGHT;

  let floor = create_static_solid(
    &mut meshes,
    &mut materials,
    Rectangle::new(width, FLOOR_HEIGHT),
    Color::hsl(150.0, 0.95, 0.7),
    Vec3::new(0.0, -height / 2.0 + FLOOR_HEIGHT / 2.0, 0.1),
    Collider::rectangle(width, FLOOR_HEIGHT),
  );
  let left_wall = create_static_solid(
    &mut meshes,
    &mut materials,
    Rectangle::new(WALL_WIDTH, height),
    Color::hsl(150.0, 0.95, 0.7),
    Vec3::new(-width / 2.0 + WALL_WIDTH / 2.0, 0.0, 0.1),
    Collider::rectangle(WALL_WIDTH, height),
  );
  let right_wall = create_static_solid(
    &mut meshes,
    &mut materials,
    Rectangle::new(WALL_WIDTH, height),
    Color::hsl(150.0, 0.95, 0.7),
    Vec3::new(width / 2.0 - WALL_WIDTH / 2.0, 0.0, 0.1),
    Collider::rectangle(WALL_WIDTH, height),
  );
  commands.spawn(floor);
  commands.spawn(left_wall);
  commands.spawn(right_wall);
}
