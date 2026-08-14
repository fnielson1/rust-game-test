use crate::components::{CoyoteTimer, HotReloadable, Player};
use crate::world_bounds::WORLD_HEIGHT;
use avian2d::prelude::{
  CoefficientCombine, Collider, Friction, GravityScale, Restitution, RigidBody, ShapeCaster,
};
use bevy::prelude::{
  Assets, Circle, Color, ColorMaterial, Commands, Dir2, Mesh, Mesh2d, MeshMaterial2d, Rectangle,
  ResMut, Transform, Vec2,
};

const PLAYER_RADIUS: f32 = 50.0;
// Multiplies the global Gravity resource for just this entity; 1.0 = unscaled, 0.0 = weightless.
const PLAYER_GRAVITY_SCALE: f32 = 2.0;
const PLAYER_FRICTION: f32 = 0.5;
// Without any Restitution, Avian defaults to 0.0 (perfectly inelastic), so the ball
// absorbs all velocity on contact and sticks to walls/floor instead of bouncing off.
const PLAYER_RESTITUTION: f32 = 0.2;
// How far below the player to look for ground. Small enough that the caster only reports
// contact when the ball is actually resting on a surface, not merely nearby.
const GROUND_CAST_DISTANCE: f32 = 4.0;

pub fn setup_player(
  mut commands: Commands,
  // Mesh storage; `meshes.add(...)` uploads geometry and returns a handle to it.
  mut meshes: ResMut<Assets<Mesh>>,
  // Material storage; `materials.add(color)` creates a solid-color material and returns a handle.
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  let spawn_y = -WORLD_HEIGHT / 2.0 + PLAYER_RADIUS * 2.0;

  // Spawn the circle, then attach the rectangle to it as a child. Child transforms are
  // relative to the parent, so rotating/moving the circle carries the rectangle with it
  // — the two entities behave as a single rigid body.
  commands
    .spawn((
      Mesh2d(meshes.add(Circle::new(PLAYER_RADIUS))),
      MeshMaterial2d(materials.add(Color::hsl(0., 0.95, 0.7))),
      Transform::from_xyz(0.0, spawn_y, 0.0),
      Player,
      // Rebuilt by the hot-patch reload; the child below goes with it, since `despawn`
      // recurses into `Children`.
      HotReloadable,
      // Dynamic: falls under gravity and collides with SolidSurface entities.
      RigidBody::Dynamic,
      Collider::circle(PLAYER_RADIUS),
      GravityScale(PLAYER_GRAVITY_SCALE),
      Friction::new(PLAYER_FRICTION).with_combine_rule(CoefficientCombine::Average),
      // Max combine rule so the ball still bounces even though the walls/floor don't
      // define their own Restitution (which would otherwise default to 0.0).
      Restitution::new(PLAYER_RESTITUTION).with_combine_rule(CoefficientCombine::Max),
      // Continuously casts a copy of the player's own shape straight down;
      // `update_grounded` reads the resulting `ShapeHits` each frame to toggle `Grounded`.
      // This is the standard Avian way to answer "is this body touching the ground" —
      // more reliable than inferring it from velocity or one-shot collision events, since
      // it works whether the ball is momentarily still, sliding, or spinning in place.
      ShapeCaster::new(
        // Make the "detector" slightly bigger to avoid issues with small bounces
        Collider::circle(PLAYER_RADIUS * 1.1),
        Vec2::ZERO,
        0.0,
        Dir2::NEG_Y,
      )
      .with_max_distance(GROUND_CAST_DISTANCE),
      // Starts "never grounded" so the coyote-time jump window can't be used before the
      // player has actually touched down once; `update_grounded` resets this on contact.
      CoyoteTimer(f32::MAX),
    ))
    .with_child((
      Mesh2d(meshes.add(Rectangle::new(4.0, PLAYER_RADIUS))),
      MeshMaterial2d(materials.add(Color::hsl(120., 0.95, 0.7))),
      Transform::from_xyz(0.0, 0.0, 0.1),
    ));
}
