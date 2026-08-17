use crate::components::{CoyoteTimer, Grounded, Player};
use avian2d::prelude::ShapeHits;
use bevy::prelude::{Commands, Entity, Query, Res, Time, With};

// How far below the ball's equator a contact has to sit before it counts as ground.
// `ShapeHitData::normal2` is the outward world-space normal on the cast shape, i.e. the
// direction from the ball's centre to the contact point: -1.0 is straight down (flat floor),
// 0.0 is level with the centre (a vertical wall), +1.0 is straight up (a ceiling). Requiring
// a negative value keeps only contacts on the lower half of the ball, and the small margin
// stops a wall grazing the equator from flickering in and out of "grounded".
const MAX_GROUND_CONTACT_NORMAL_Y: f32 = -0.1;

/// Keeps `Grounded` and `CoyoteTimer` in sync with the player's `ShapeCaster` (added in
/// `setup_player`). Must run before `player_input` so a jump this frame sees up-to-date
/// grounded state.
///
/// The caster reports contact in every direction (see `setup_player` for why), so touching a
/// ceiling or a wall would otherwise refill the jump. Only contacts on the underside of the
/// ball — the part that could actually be resting on something — count as ground.
pub fn update_grounded(
  mut commands: Commands,
  time: Res<Time>,
  mut query: Query<(Entity, &ShapeHits, &mut CoyoteTimer), With<Player>>,
) {
  for (entity, hits, mut coyote_timer) in &mut query {
    let grounded = hits
      .iter()
      .any(|hit| hit.normal2.y <= MAX_GROUND_CONTACT_NORMAL_Y);

    if grounded {
      coyote_timer.0 = 0.0;
      commands.entity(entity).insert(Grounded);
    } else {
      coyote_timer.0 += time.delta_secs();
      commands.entity(entity).remove::<Grounded>();
    }
  }
}
