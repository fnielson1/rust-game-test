use crate::components::{HotReloadable, SolidSurface};
use avian2d::prelude::{CoefficientCombine, Collider, Friction, Restitution, RigidBody};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::prelude::{ColorMaterial, MeshMaterial2d, Transform};

const SOLID_FRICTION: f32 = 10.0;
const SOLID_RESTITUTION: f32 = 0.5;

pub fn create_static_solid(
  // Mesh storage; `meshes.add(...)` uploads geometry and returns a handle to it.
  meshes: &mut Assets<Mesh>,
  // Material storage; `materials.add(color)` creates a solid-color material and returns a handle.
  materials: &mut Assets<ColorMaterial>,
  mesh: impl Into<Mesh>,
  color: Color,
  // Full transform rather than a translation: level segments lie at arbitrary angles, and
  // rotation is what makes that expressible. Avian applies this same rotation to `collider`,
  // so the collision shape follows the drawn mesh without being constructed differently.
  transform: Transform,
  // Collision shape matching `mesh`, so the surface can be hit by dynamic rigid bodies.
  collider: Collider,
) -> (
  Mesh2d,
  MeshMaterial2d<ColorMaterial>,
  Transform,
  SolidSurface,
  HotReloadable,
  RigidBody,
  Friction,
  Restitution,
  Collider,
) {
  (
    Mesh2d(meshes.add(mesh)),
    MeshMaterial2d(materials.add(color)),
    transform,
    SolidSurface,
    // Every level is built from these, so marking them here is what makes a new level
    // hot-reloadable without touching `hotpatch_reload`.
    HotReloadable,
    // Static: never moves, but dynamic bodies collide with and rest on it.
    RigidBody::Static,
    Friction::new(SOLID_FRICTION).with_combine_rule(CoefficientCombine::Average),
    Restitution::new(SOLID_RESTITUTION).with_combine_rule(CoefficientCombine::Max),
    collider,
  )
}
