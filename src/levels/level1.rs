use crate::create_solid_surface::create_solid_surface;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::mesh::Mesh;
use bevy::prelude::{ColorMaterial, Commands, Query, Rectangle, ResMut, Vec3, With};
use bevy::window::{PrimaryWindow, Window};

const FLOOR_HEIGHT: f32 = 20.0;

pub fn level1(
  mut commands: Commands,
  // Mesh storage; `meshes.add(...)` uploads geometry and returns a handle to it.
  meshes: ResMut<Assets<Mesh>>,
  // Material storage; `materials.add(color)` creates a solid-color material and returns a handle.
  materials: ResMut<Assets<ColorMaterial>>,
  windows: Query<&Window, With<PrimaryWindow>>,
) {
  // 2D world units map 1:1 to logical pixels by default, so the primary window's logical
  // width/height double as the visible world bounds.
  let window = windows
    .single()
    .expect("primary window should exist by Startup");
  let width = window.width();
  let height = window.height();

  let floor = create_solid_surface(
    meshes,
    materials,
    Rectangle::new(width, FLOOR_HEIGHT),
    Color::hsl(150.0, 0.95, 0.7),
    Vec3::new(0.0, -height / 2.0 + FLOOR_HEIGHT / 2.0, 0.1),
  );
  commands.spawn(floor);
}
