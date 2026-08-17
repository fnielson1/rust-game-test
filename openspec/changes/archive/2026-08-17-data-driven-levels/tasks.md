## 1. Dependencies and build setup

- [x] 1.1 Add `serde` (with the `derive` feature) and `serde_json` as direct dependencies in `Cargo.toml`
- [x] 1.2 Add `bevy/file_watcher` to the `hotpatch` feature list so asset hot-reload is on in live-editing builds only
- [x] 1.3 Create `assets/levels/` and add a copy directive so `assets/` is served from `dist/` in the wasm build
      (declared in `index.html` as `rel="copy-dir"` — Trunk has no Trunk.toml equivalent)
- [x] 1.4 Verify both builds still compile before any behavior changes
      (`cargo check` native + `--target wasm32-unknown-unknown`; runtime launch covered in group 7)

## 2. Level file format

- [x] 2.1 Define `Segment` deriving `Deserialize`, with `start: [f32; 2]`, `end: [f32; 2]`, `thickness: f32`
      (`LevelData` folded into `LevelAsset` — on-disk and in-memory shapes are identical, so a
      separate intermediate type would only add a copy)
- [x] 2.2 Write `assets/levels/level1.level.json` re-expressing the current floor and two walls as segments, matching today's positions derived from `WORLD_WIDTH`/`WORLD_HEIGHT`
- [x] 2.3 Add at least one sloped segment and one gap to the level file so the new capabilities are exercised from the first run

## 3. Asset type and loader

- [x] 3.1 Create `src/levels/level_asset.rs` with `LevelAsset` deriving `Asset` and `TypePath`, wrapping the parsed segment list
- [x] 3.2 Implement `LevelLoader` as an `AssetLoader` that reads bytes and parses with `serde_json::from_slice`, registered for the `.level.json` extension
- [x] 3.3 Return a typed loader error on parse failure so the message names the file and the JSON error
- [x] 3.4 Register the asset type and loader with the `App` via `init_asset::<LevelAsset>()` and
      `init_asset_loader::<LevelLoader>()` (loader is `Default`, so no instance to pass), in a new `LevelPlugin`

## 4. Rotated static solids

- [x] 4.1 Change `create_static_solid` to take a `Transform` instead of a `Vec3` translation, replacing the internal `Transform::from_xyz` call
- [x] 4.2 Update the existing call sites — all three lived in the rewritten `level1`, so `spawn_level` is now the only caller
- [x] 4.3 Add a helper that converts a `Segment` into its mesh `Rectangle::new(length, thickness)`, midpoint translation, and `Quat::from_rotation_z(atan2(dy, dx))` rotation
- [x] 4.4 Confirm `Collider::rectangle(length, thickness)` picks up the entity's `Transform` rotation
      (verified live: tilting the floor in the JSON left the player resting *on* the incline, not at the old horizontal height)

## 5. Spawning a loaded level

- [x] 5.1 Add a `LevelHandle` resource holding the `Handle<LevelAsset>` for the current level
- [x] 5.2 Rewrite `src/levels/level1.rs` to load `levels/level1.level.json` into `LevelHandle` and request a re-spawn, instead of hardcoding geometry
- [x] 5.3 Write `spawn_level` in `src/levels/spawn_level.rs`: despawn the previous level's segment entities, then spawn one static solid per valid segment
- [x] 5.4 Skip degenerate segments (coincident endpoints, thickness `<= 0`) with a warning naming the segment index and the reason, and continue with the rest
- [x] 5.5 Log an error naming the attempted path when the level asset fails to load or is missing, without panicking

## 6. Reacting to changes

- [x] 6.1 Run `spawn_level` on `AssetEvent::<LevelAsset>::LoadedWithDependencies` for the handle in `LevelHandle`
- [x] 6.2 Run `spawn_level` on `AssetEvent::<LevelAsset>::Modified` so saving the file rebuilds the level live
- [x] 6.3 Ensure the re-spawn path clears previous segments first, so repeated saves never accumulate duplicate surfaces
- [x] 6.4 Verify the hot-patch path still rebuilds the level
      (patched `spawn_level` under `dx serve`: `Hot-patching: src/levels/spawn_level.rs took 2135ms` ->
      `level rebuilt: despawned 0, spawned 6`. `despawned 0` is correct — `despawn_world` clears the
      previous segments before `spawn_level` runs)

## 7. Verification

- [x] 7.1 Run the game and confirm the level matches the JSON — floor, wall, ramp and gap all render;
      `level rebuilt: despawned 0, spawned 6`, and the floor's edge lands at exactly the pixel world x=150 maps to
- [x] 7.2 Confirm the player rests on a horizontal segment, collides with the sloped segment, and falls through the gap
      (rest + slope confirmed visually; the fall confirmed by the player ending up below world y=-480.
      `Grounded` itself was not read directly — it has no log and no test harness exists)
- [x] 7.3 Edit a segment's endpoints, save, confirm the surface moves without a restart
      (`Reloaded levels/level1.level.json` -> `level rebuilt: despawned 6, spawned 6`)
- [x] 7.4 Change the segment list while running and confirm no duplicates accumulate
      (6->1 logged `despawned 6, spawned 1`; 1->6 logged `despawned 1, spawned 6`; repeated saves each
      despawned exactly what the previous rebuild spawned)
- [x] 7.5 Apply a code hot patch and confirm the level is still present and not duplicated
      (changed the segment hue to 260 and the rebuilt level came back purple — proving the geometry was
      respawned from the *patched* code, not left over from before the patch)
- [x] 7.6 Corrupt the JSON — logs ``could not parse level file `levels/level1.level.json`: expected value at line 1 column 42``
      and the game kept running
- [x] 7.7 Rename the level file — logs `Path not found: ./assets/levels/level1.level.json` at startup, zero panics
- [x] 7.8 Build and serve the wasm target and confirm the same level file loads and renders there
      (found a real wasm-only bug: trunk's SPA fallback answered bevy's `.meta` probe with `index.html`/200
      instead of 404, failing every asset load. Fixed with `no_spa = true` in `Trunk.toml`; the browser then
      rendered the full level — both walls, both floor segments, the gap, the ramp, and the player at rest)
- [x] 7.9 `cargo clippy` and `cargo fmt` clean on both the default and `--features hotpatch` configurations
      (the repo's `shadow_reuse` deny caught `let Some(x) = x`; unwrapped bindings renamed)
