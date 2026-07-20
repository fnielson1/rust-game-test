use crate::components::{Grounded, Player};
use avian2d::prelude::ShapeHits;
use bevy::prelude::{Commands, Entity, Query, With};

/// Keeps the `Grounded` marker in sync with the player's downward `ShapeCaster` (added in
/// `setup_player`). Must run before `player_input` so a jump this frame sees an up-to-date
/// grounded state.
pub fn update_grounded(mut commands: Commands, query: Query<(Entity, &ShapeHits), With<Player>>) {
  for (entity, hits) in &query {
    if hits.iter().next().is_some() {
      commands.entity(entity).insert(Grounded);
    } else {
      commands.entity(entity).remove::<Grounded>();
    }
  }
}
