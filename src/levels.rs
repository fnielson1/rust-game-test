pub mod level1;
pub mod level_asset;
pub mod spawn_level;

use bevy::asset::AssetApp;
use bevy::prelude::{App, IntoScheduleConfigs, Plugin, Startup, Update};
use level_asset::{LevelAsset, LevelLoader};
use spawn_level::{
  LevelRespawn, report_level_load_failures, respawn_on_level_asset_change, spawn_level,
};

/// Registers the level file format and the systems that build levels from it.
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
  fn build(&self, app: &mut App) {
    app
      .init_asset::<LevelAsset>()
      .init_asset_loader::<LevelLoader>()
      .init_resource::<LevelRespawn>()
      .add_systems(Startup, level1::level1)
      // Chained so an asset change and the rebuild it triggers land in the same frame rather
      // than leaving one frame with the old geometry.
      .add_systems(Update, (respawn_on_level_asset_change, spawn_level).chain())
      .add_systems(Update, report_level_load_failures);
  }
}
