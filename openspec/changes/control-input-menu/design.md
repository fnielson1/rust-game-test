## Context

`src/player/player_input.rs` reads `KeyCode::Space`/`ArrowUp` (jump) and `KeyCode::ArrowLeft`/`ArrowRight` (rotate) directly off the `ButtonInput<KeyCode>` resource every frame. There is no game-state concept (`bevy::state`) yet, no UI beyond raw shape rendering, and no persistence layer — this is a single-player local prototype (`Cargo.toml`: `bevy = "0.19.0"`, `avian2d = "0.7.0"`, built for both native and `wasm32-unknown-unknown` via `trunk`). This change adds the first UI screen and the first app-level state machine to the project.

## Goals / Non-Goals

**Goals:**
- Replace hardcoded `KeyCode` reads in gameplay code with lookups against a `KeyBindings` resource.
- Ship a functional settings menu: open/close, view current bindings, rebind by click-then-press-a-key, reject collisions.
- Pause gameplay (input processing + physics stepping) while the menu is open, resume cleanly on close.

**Non-Goals:**
- Persisting bindings across page reloads/process restarts (localStorage/file I/O). Bindings live in memory for the process lifetime; a follow-up change can add persistence once the in-memory shape is proven.
- Gamepad/touch rebinding — keyboard only, matching the game's current input surface.
- Multiple simultaneous keys per action (today's `Space` *or* `ArrowUp` jump alias collapses to a single bound key per action; see Decisions).
- General pause menu features (quit, resume-only-button, audio settings, etc.) — this menu's only job is control bindings, though it's built so a later change can extend it.

## Decisions

### One key per action, not a key set
**Decision**: `KeyBindings` maps `InputAction -> KeyCode` (one key), not `InputAction -> HashSet<KeyCode>`.
**Why**: The rebind UI ("click a row, press a key") is unambiguous with a single bound key — there's exactly one thing to show and one thing to replace. A set requires add/remove-individual-key UI that the proposal doesn't ask for.
**Consequence**: `Jump`'s current `Space`-or-`ArrowUp` alias collapses to `Space` only by default. `ArrowUp` stops jumping unless the player rebinds `Jump` to it. This is called out as a minor behavior change, not a regression worth blocking on.
**Alternative considered**: Keep multi-key aliases and only let the menu edit a "primary" key. Rejected — adds a hidden second key the UI never shows, which is confusing.

### Actions as a `#[derive(Component)]`-free plain enum, keyed by a component-less resource
**Decision**: `InputAction` is a plain `enum { Jump, RotateLeft, RotateRight }` deriving `Clone, Copy, PartialEq, Eq, Hash`. `KeyBindings` is a `Resource` wrapping `HashMap<InputAction, KeyCode>` with a `Default` impl providing today's defaults (minus the dropped alias) and a `bound_key(&self, action) -> KeyCode` accessor.
**Why**: Matches the existing codebase's style (plain components/resources, no external input-mapping crate). Keeps `player_input.rs`'s diff small: swap `keys.pressed(KeyCode::ArrowLeft)` for `keys.pressed(bindings.bound_key(InputAction::RotateLeft))`.
**Alternative considered**: Pull in `leafwing-input-manager`. Rejected — new dependency for a 3-action prototype is disproportionate; Non-Goals already exclude gamepad support that would justify it.

### `bevy::state`-based `AppState` gates both input and physics
**Decision**: Add `#[derive(States)]enum AppState { Playing, Menu }`, `init_state::<AppState>()` (default `Playing`). `player_input` runs under `.run_if(in_state(AppState::Playing))`. Opening the menu also calls `Time::<Physics>::pause()` (avian2d's `PhysicsTime` trait, `avian2d::prelude::Physics`/`Time<Physics>`); closing calls `.unpause()`.
**Why**: `bevy::state` is the standard, idiomatic way to gate systems in Bevy 0.19 and composes with `run_if`/`OnEnter`/`OnExit` for spawning and despawning the menu UI. Avian2d exposes exactly this pause hook (`Time<Physics>::pause()/unpause()/is_paused()`) so physics stepping stops without touching gravity or velocities — objects don't drift or reset, they just stop simulating.
**Alternative considered**: A plain `bool` "paused" resource checked ad hoc in every system. Rejected — `bevy::state` gives `OnEnter(AppState::Menu)`/`OnExit(AppState::Menu)` for UI spawn/despawn for free, which a bool resource doesn't.

### Rebind capture: "awaiting key" resource + `keyboard_input` event scan
**Decision**: Clicking a binding row sets `RebindRequest(Some(InputAction))` (a `Resource`). A dedicated system runs only while `RebindRequest` is `Some`, reads `ButtonInput<KeyCode>::get_just_pressed()`, and on the first key: if that `KeyCode` is already bound to a *different* action, set an inline `RebindError` message and leave bindings unchanged; otherwise write the new binding into `KeyBindings` and clear `RebindRequest`. `Escape` while awaiting a key cancels the capture (does not close the menu) instead of binding `Escape` itself.
**Why**: Keeps capture logic in one system instead of scattering "am I capturing?" checks across button-click handlers. Collision check is a simple reverse lookup over the small (3-entry) map — no need for an index structure.
**Alternative considered**: Bind immediately on any key including duplicates, last-write-wins. Rejected — proposal explicitly calls for rejecting collisions with a message.

### Menu built with `bevy_ui`, no new dependency
**Decision**: Menu is a `Node`-based UI tree (`bevy_ui`, already part of `DefaultPlugins`), spawned in `OnEnter(AppState::Menu)` and despawned in `OnExit(AppState::Menu)` (despawn via a marker component + recursive despawn, matching Bevy 0.19's `entity.despawn()` on the root cascading to children).
**Why**: Zero new dependencies, consistent with the rest of the project (`Mesh2d`/`ColorMaterial` used directly rather than a UI framework), and sufficient for a 3-row list + labels.

## Risks / Trade-offs

- **[Risk]** Dropping the `ArrowUp`-jump alias changes existing feel/muscle memory for whoever has been playtesting → **Mitigation**: default `Jump` stays on `Space` (the primary key), and it's a one-click rebind if `ArrowUp` is wanted back; called out explicitly in the proposal and here rather than silently changed.
- **[Risk]** No persistence means every reload silently discards custom bindings, which can look like a bug ("I set W to jump and it's gone") → **Mitigation**: Non-Goal is explicit in the proposal; follow-up persistence change is named as the next step.
- **[Risk]** Pausing only `Time<Physics>` and gating `player_input` still leaves `camera_follow` running every frame while paused → **Mitigation**: acceptable — camera easing toward a stationary player is a no-op once it catches up, and freezing the camera too has no gameplay benefit worth the extra run-condition wiring.
- **[Trade-off]** Single-key-per-action drops a working alias (see Decisions) in exchange for a simpler, unambiguous rebind UI.

## Open Questions

- None blocking. Persistence approach (browser `localStorage` via `web-sys`/`gloo-storage` vs. a native-only save file) is deferred to when a persistence change is actually proposed.
