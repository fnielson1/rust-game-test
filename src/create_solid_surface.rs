use crate::components::SolidSurface;
use avian2d::prelude::{Collider, RigidBody};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::{ColorMaterial, MeshMaterial2d, ResMut, Transform, Vec3};

pub fn create_solid_surface(
  // Mesh storage; `meshes.add(...)` uploads geometry and returns a handle to it.
  mut meshes: ResMut<Assets<Mesh>>,
  // Material storage; `materials.add(color)` creates a solid-color material and returns a handle.
  mut materials: ResMut<Assets<ColorMaterial>>,
  mesh: impl Into<Mesh>,
  color: Color,
  transform: Vec3,
  // Collision shape matching `mesh`, so the surface can be hit by dynamic rigid bodies.
  collider: Collider,
) -> (
  Mesh2d,
  MeshMaterial2d<ColorMaterial>,
  Transform,
  SolidSurface,
  RigidBody,
  Collider,
) {
  (
    Mesh2d(meshes.add(mesh)),
    MeshMaterial2d(materials.add(color)),
    // Rectangle::new is centered on its Transform, so push it down by half the window height
    // (screen bottom) plus half its own thickness (so its bottom edge lands on the window edge).
    Transform::from_xyz(transform.x, transform.y, transform.z),
    SolidSurface,
    // Static: never moves, but dynamic bodies collide with and rest on it.
    RigidBody::Static,
    collider,
  )
}
