## Context

Today a level is a Rust function. `level1` builds three `create_static_solid` bundles from
`WORLD_WIDTH`/`WORLD_HEIGHT` and spawns them; `create_static_solid` takes a `Vec3` translation
and calls `Transform::from_xyz`, so every surface it can produce is axis-aligned. There is no
`assets/` directory in the repo at all.

Two existing mechanisms constrain the design:

- **Hot patching.** `hotpatch_reload::HotPatchReloadPlugin` runs `(despawn_world, setup_player,
  level1).chain()` whenever a `HotPatched` message fires, and `despawn_world` deletes everything
  carrying `HotReloadable`. The invariant it documents is strict: an entity carries
  `HotReloadable` if and only if one of those re-run systems will spawn it again.
- **Two build targets.** Native runs through `dx serve`/`cargo`, wasm through trunk. Anything
  that reads a file has to work on both, which rules out `std::fs` in the spawn path.

The user has settled three questions up front: load through `AssetServer` with a custom
`AssetLoader`; describe segments as two endpoints plus a thickness; and replace `level1`'s
hardcoded geometry rather than adding a parallel path.

## Goals / Non-Goals

**Goals:**
- A level is a JSON file listing any number of segments, at any angle, with gaps expressed by
  omission.
- Segments are collidable and behave exactly like today's static solids (same friction,
  restitution, `SolidSurface` marker) so the player can stand, slide, and land on them.
- Editing a level's JSON and saving rebuilds it in the running game — no recompile, no restart.
- The same level file works on native and wasm.
- A malformed or missing level file degrades to a clear log message, not a panic or a silent
  empty world.

**Non-Goals:**
- An in-game or external level editor. Levels are hand-edited JSON for now.
- Non-line geometry: curves, polygons, filled regions, one-way platforms, moving platforms.
- Per-segment material overrides beyond color, or per-segment physics tuning. Friction and
  restitution stay global constants in `create_static_solid`.
- Level progression, transitions, or more than one level being resident at a time.
- Backwards compatibility with the current hardcoded `level1` shape as *code*; it is re-expressed
  as data.

## Decisions

### Segments are endpoints + thickness, rendered as a rotated rectangle

A segment is `{ "start": [x, y], "end": [x, y], "thickness": f32 }`. From that:

- `length = (end - start).length()`, `angle = atan2(dy, dx)`
- mesh: `Rectangle::new(length, thickness)`
- transform: translation at the midpoint `(start + end) / 2.0`, rotation `Quat::from_rotation_z(angle)`
- collider: `Collider::rectangle(length, thickness)` — avian applies the entity's `Transform`
  rotation to the collider, so no separate rotated-collider construction is needed.

*Why endpoints over origin+angle+length:* a gap becomes a non-decision. The author places two
segments whose endpoints don't meet, and the gap is whatever is between them. With
origin+angle+length, expressing "the next segment starts 100 units past where the last one
ended" requires the author to do trigonometry by hand.

*Why a rotated rectangle over `Collider::segment`:* a true segment collider is zero-thickness,
which lets fast bodies tunnel through and gives the renderer nothing to draw. Thickness is also
a property level authors will want anyway.

### Zero-length segments are rejected, not clamped

`atan2(0, 0)` is 0 and a zero-length rectangle is a degenerate collider that avian will happily
accept and then behave strangely around. A segment whose endpoints coincide (or whose thickness
is `<= 0`) is skipped with a warning naming its index, and the rest of the level still loads.
Partial load beats an all-or-nothing failure while hand-editing.

### `create_static_solid` takes a full `Transform`, not a `Vec3`

Its current signature cannot express rotation. Rather than adding a fourth positional angle
parameter, the translation parameter becomes a `Transform`. Callers that don't rotate pass
`Transform::from_xyz(...)` — the same expression that's inside the function today — so the
change is mechanical and the rotation case needs no special path.

*Alternative considered:* a separate `create_rotated_static_solid`. Rejected: two functions that
differ only in whether one field is identity, and the level spawner would be the only caller of
one of them.

### `LevelAsset` + custom `AssetLoader`, keyed off `AssetEvent`

`LevelAsset` is `#[derive(Asset, TypePath)]` wrapping the deserialized segment list. A
`LevelLoader` implementing `AssetLoader` reads the bytes and runs `serde_json::from_slice`,
registered for the `.level.json` extension so it can't collide with any other JSON the project
later loads.

A `LevelHandle` resource holds the `Handle<LevelAsset>`. One system reacts to
`AssetEvent::<LevelAsset>::{LoadedWithDependencies, Modified}` for that handle and re-spawns.

*Why not `include_str!`:* it was on the table and rejected by the user, but the reason is worth
recording — embedding means every level tweak is a recompile, which is exactly the loop this
change exists to remove.

### Re-spawn is one function, shared by the asset path and the hot-patch path

This is the subtle part. `hotpatch_reload` re-runs `level1` on every patch. If `level1` becomes
"call `asset_server.load()`", re-running it on a patch loads an already-loaded handle, fires no
new `AssetEvent`, and the level never comes back — `despawn_world` will have just deleted it.

So the work splits:

- `level1` (Startup, and re-run on hot patch): loads the handle into the `LevelHandle` resource,
  and unconditionally requests a re-spawn.
- `spawn_level`: despawns existing segments, then spawns one static solid per segment from the
  currently-loaded `LevelAsset`. Runs when a re-spawn is requested *or* when the asset changes.

Segments keep the `HotReloadable` marker (they come from `create_static_solid`, which adds it),
so `despawn_world` still clears them and the existing invariant holds unchanged.

*Alternative considered:* have the hot-patch path force an asset reload. Rejected: it makes every
unrelated code patch re-read the file from disk, and it couples the patch path to loader
internals for no gain.

### Asset hot-reload needs the `file_watcher` feature, native only

Bevy only watches asset files when the `file_watcher` feature is on. It is not in the default
feature set, and it does nothing on wasm (no filesystem to watch). It also carries a `notify`
dependency and a background thread, which a release build shouldn't pay for.

It therefore rides along with the existing `hotpatch` feature rather than becoming a fourth
build mode — that feature already means "this is the live-editing build," and it's already
native-only by construction. On wasm, level JSON is read once at load; editing it means a page
reload, which is the same loop trunk already provides.

### Asset paths and the two build systems

- Native: assets resolve relative to the executable, which for `dx` lives under
  `target/dx/rust-game-test/debug/linux/app/`. The `hot` script already sets
  `BEVY_ASSET_ROOT="."` to point Bevy back at the repo root, so `assets/levels/` resolves. This
  is the first change that makes that env var load-bearing — it has been inert until now.
- Wasm: `Trunk.toml` needs a copy directive so `assets/` lands in `dist/` and is served
  alongside the wasm bundle.

## Risks / Trade-offs

- **The player spawns before the ground exists.** Assets load asynchronously, so `Startup`
  finishes with an empty world and the player begins falling. → The player starts well above the
  floor and gravity is 400; a few frames of fall is invisible. If it turns out not to be, the fix
  is to gate physics on the level being loaded, not to make loading synchronous. Flagged rather
  than pre-solved.

- **`BEVY_ASSET_ROOT` becomes required for native runs.** Launching any way that doesn't set it
  (RustRover's Cargo run button, a bare `cargo run`) will now fail to find levels, where before
  the level was compiled in and always worked. → The failure must be a clear log line naming the
  path it tried, not an empty screen. Worth noting in the run configs.

- **Partial loads can hide typos.** Skipping a bad segment and loading the rest means a
  fat-fingered coordinate produces a level that looks almost right. → Every skipped segment logs
  a warning with its index and the reason.

- **`create_static_solid`'s signature change touches every caller.** Small blast radius today
  (three call sites in `level1`, which is being rewritten anyway), but it is a breaking change to
  a shared helper. → Do it in one commit with the level rewrite so the tree is never half-migrated.

- **`file_watcher` tied to the `hotpatch` feature couples two independent things.** Someone who
  wants live level editing without subsecond's per-system-call indirection has to take both. →
  Accepted for now; the combination is what the `hot` script does anyway. Splitting into a
  separate feature is easy later if the pairing chafes.

- **No schema versioning.** A future format change silently breaks old files. → Out of scope
  while there is exactly one level file in the repo, but a `"version"` field is cheap to add
  before there are many.
