use crate::components::{CoyoteTimer, Grounded, Player};
use avian2d::prelude::ShapeHits;
use bevy::prelude::{Commands, Entity, Query, Res, Time, With};

/// Keeps `Grounded` and `CoyoteTimer` in sync with the player's downward `ShapeCaster` (added
/// in `setup_player`). Must run before `player_input` so a jump this frame sees up-to-date
/// grounded state.
pub fn update_grounded(
  mut commands: Commands,
  time: Res<Time>,
  mut query: Query<(Entity, &ShapeHits, &mut CoyoteTimer), With<Player>>,
) {
  for (entity, hits, mut coyote_timer) in &mut query {
    if hits.iter().next().is_some() {
      coyote_timer.0 = 0.0;
      commands.entity(entity).insert(Grounded);
    } else {
      coyote_timer.0 += time.delta_secs();
      commands.entity(entity).remove::<Grounded>();
    }
  }
}
