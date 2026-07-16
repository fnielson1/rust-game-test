use crate::components::Player;
use avian2d::prelude::{CoefficientCombine, Collider, Friction, GravityScale, RigidBody};
use bevy::prelude::{
  Assets, Circle, Color, ColorMaterial, Commands, Mesh, Mesh2d, MeshMaterial2d, Query, Rectangle,
  ResMut, Transform, With,
};
use bevy::window::{PrimaryWindow, Window};

const PLAYER_RADIUS: f32 = 50.0;
// Multiplies the global Gravity resource for just this entity; 1.0 = unscaled, 0.0 = weightless.
const PLAYER_GRAVITY_SCALE: f32 = 1.0;
const PLAYER_FRICTION: f32 = 0.5;

pub fn setup_player(
  mut commands: Commands,
  // Mesh storage; `meshes.add(...)` uploads geometry and returns a handle to it.
  mut meshes: ResMut<Assets<Mesh>>,
  // Material storage; `materials.add(color)` creates a solid-color material and returns a handle.
  mut materials: ResMut<Assets<ColorMaterial>>,
  windows: Query<&Window, With<PrimaryWindow>>,
) {
  // 2D world units map 1:1 to logical pixels, so the primary window's logical height
  // doubles as the visible world bounds (see level1.rs).
  let window = windows
    .single()
    .expect("primary window should exist by Startup");
  let spawn_y = -window.height() / 2.0 + PLAYER_RADIUS * 2.0;

  // Spawn the circle, then attach the rectangle to it as a child. Child transforms are
  // relative to the parent, so rotating/moving the circle carries the rectangle with it
  // — the two entities behave as a single rigid body.
  commands
    .spawn((
      Mesh2d(meshes.add(Circle::new(PLAYER_RADIUS))),
      MeshMaterial2d(materials.add(Color::hsl(0., 0.95, 0.7))),
      Transform::from_xyz(0.0, spawn_y, 0.0),
      Player,
      // Dynamic: falls under gravity and collides with SolidSurface entities.
      RigidBody::Dynamic,
      Collider::circle(PLAYER_RADIUS),
      GravityScale(PLAYER_GRAVITY_SCALE),
      Friction::new(PLAYER_FRICTION).with_combine_rule(CoefficientCombine::Average),
    ))
    .with_child((
      Mesh2d(meshes.add(Rectangle::new(4.0, 50.0))),
      MeshMaterial2d(materials.add(Color::hsl(120., 0.95, 0.7))),
      Transform::from_xyz(0.0, 0.0, 0.1),
    ));
}
