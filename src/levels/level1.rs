use crate::levels::level_asset::LevelAsset;
use crate::levels::spawn_level::{LevelHandle, LevelRespawn};
use bevy::prelude::{AssetServer, Commands, Res, ResMut};

/// Path of the level file, relative to the asset root.
///
/// On native, the asset root is wherever `BEVY_ASSET_ROOT` points -- the `hot` script sets it to
/// the repo root, since the dx-built binary lives several directories deep under `target/dx/`.
/// On wasm, trunk copies `assets/` into `dist/` (see the `copy-dir` link in `index.html`) and it
/// is served from there.
const LEVEL1_PATH: &str = "levels/level1.level.json";

/// Loads level 1 and asks for it to be built.
///
/// Deliberately does no spawning of its own. Geometry comes from the file, and everything that
/// creates entities lives in [`crate::levels::spawn_level::spawn_level`] so that the startup
/// path, the file-changed path, and the hot-patch path all rebuild the level identically.
///
/// `AssetServer::load` is idempotent for an already-loaded path, which is exactly why the
/// explicit respawn request below is needed: when the hot-patch reload re-runs this system, the
/// load is a no-op and fires no asset event, so nothing else would trigger the rebuild.
pub fn level1(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut respawn: ResMut<LevelRespawn>,
) {
  commands.insert_resource(LevelHandle(asset_server.load::<LevelAsset>(LEVEL1_PATH)));
  respawn.request();
}
