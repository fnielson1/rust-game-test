## Why

Level geometry is hardcoded in Rust: `level1.rs` spawns exactly three axis-aligned rectangles
(a floor and two walls) built from `WORLD_WIDTH`/`WORLD_HEIGHT` constants. Adding a level means
writing a new function and recompiling, and the shape vocabulary is limited to axis-aligned
boxes — there is no way to express a sloped surface or a gap to jump across, which are the two
things a platformer level is actually made of.

Moving geometry into JSON makes levels data instead of code: any number of line segments, at any
angle, with gaps wherever the author leaves one.

## What Changes

- Add a JSON level format describing a level as a list of line segments, each given as two
  endpoints plus a thickness. Angle is implied by the endpoints; a gap is simply the absence of
  a segment between two others.
- Add a `LevelAsset` type and a custom Bevy `AssetLoader` that deserializes those JSON files, so
  levels load through `AssetServer` and work identically on native and wasm.
- Spawn each segment as an existing `create_static_solid` body: a `Rectangle` mesh of
  `length x thickness`, rotated to the segment's angle, with a matching rotated `Collider`.
- Re-spawn the level when its JSON asset changes on disk, so editing a level and saving rebuilds
  it in the running game without a restart or a recompile.
- **BREAKING**: `level1` stops hardcoding its geometry and instead loads
  `assets/levels/level1.json`. The current floor and two walls are re-expressed as segments in
  that file, so the level looks the same on first run.
- Add an `assets/` directory (the repo has none today) and make sure the wasm/trunk build ships
  it.

## Capabilities

### New Capabilities
- `level-format`: The JSON level file format — its schema, the meaning of a segment's endpoints
  and thickness, how gaps are expressed, and how malformed or missing files are handled.
- `level-loading`: Loading a level asset into the running game — spawning collidable, rotated
  segments from the parsed data, and re-spawning when the underlying file changes.

### Modified Capabilities
<!-- None. `input-bindings` and `settings-menu` are untouched; no existing spec's requirements
     change. Level geometry has never had a spec, so it arrives as new capabilities above. -->

## Impact

- **Code**: `src/levels/level1.rs` (rewritten to load rather than hardcode), `src/levels.rs`
  (new modules), new `src/levels/level_asset.rs` and `src/levels/spawn_level.rs`.
  `src/create_static_solid.rs` gains rotation support — it currently takes a `Vec3` translation
  and builds `Transform::from_xyz`, which cannot express an angled segment.
- **Hot patching**: `hotpatch_reload.rs` re-runs `level1` on every `HotPatched` message. That
  system becomes a load rather than a spawn, so the rebuild path has to route through the same
  re-spawn logic the asset-change path uses, or hot patches will stop rebuilding the level.
- **Dependencies**: adds `serde` (derive) and `serde_json`. Bevy already depends on both
  transitively, but they need to be direct dependencies to be used.
- **Assets**: introduces `assets/`, which affects the native run (`BEVY_ASSET_ROOT="."` in the
  `hot` script already points at the repo root) and the trunk/wasm build (`Trunk.toml` needs a
  copy directive so `assets/` reaches `dist/`).
- **Startup ordering**: assets load asynchronously, so the level is no longer guaranteed to exist
  at the end of `Startup`. The player currently spawns in `setup` at a fixed position; it may
  fall a frame or two before the ground exists.
