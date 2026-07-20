## 1. Input bindings resource

- [x] 1.1 Add `src/input/mod.rs` with `pub enum InputAction { Jump, RotateLeft, RotateRight }` (`Clone, Copy, PartialEq, Eq, Hash, Debug`)
- [x] 1.2 Add `KeyBindings` resource (`HashMap<InputAction, KeyCode>`) with a `Default` impl: `Jump -> Space`, `RotateLeft -> ArrowLeft`, `RotateRight -> ArrowRight`
- [x] 1.3 Add `KeyBindings::bound_key(&self, action: InputAction) -> KeyCode` and `KeyBindings::action_for_key(&self, key: KeyCode) -> Option<InputAction>` (reverse lookup, used for collision detection)
- [x] 1.4 Add `KeyBindings::rebind(&mut self, action: InputAction, key: KeyCode) -> Result<(), InputAction>` — returns `Err(other_action)` if `key` is already bound to a different action, otherwise updates the binding and returns `Ok(())`
- [x] 1.5 Register `KeyBindings` as an app resource (`init_resource`) in `src/main.rs`
- [x] 1.6 Update `src/player/player_input.rs` to take `Res<KeyBindings>` and use `keys.pressed(bindings.bound_key(InputAction::RotateLeft))` / `RotateRight` / `just_pressed(bindings.bound_key(InputAction::Jump))` in place of the literal `KeyCode`s (drop the `ArrowUp` jump alias per design.md)

## 2. App state and physics pause

- [x] 2.1 Add `#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)] pub enum AppState { #[default] Playing, Menu }` (new `src/app_state.rs` or alongside `components.rs`)
- [x] 2.2 Register state in `src/main.rs` via `app.init_state::<AppState>()`
- [x] 2.3 Gate `player_input` in the `Update` schedule with `.run_if(in_state(AppState::Playing))`
- [x] 2.4 Add a system that toggles `AppState` between `Playing` and `Menu` on `Escape` `just_pressed` (only when not mid-rebind-capture; see 3.5) and drives `Time::<Physics>::pause()`/`unpause()` (avian2d `PhysicsTime` trait) on the corresponding transition

## 3. Rebind capture logic

- [x] 3.1 Add `RebindRequest(Option<InputAction>)` resource, `init_resource` in `src/main.rs`, default `None`
- [x] 3.2 Add `RebindError(Option<String>)` resource for the last-rejected-rebind message, `init_resource`, default `None`
- [x] 3.3 Add a system (runs only in `AppState::Menu`) that, when `RebindRequest` is `Some(action)`, reads `ButtonInput<KeyCode>::get_just_pressed()`:
  - if the pressed key is `Escape`, clear `RebindRequest` (cancel, do not rebind, do not close menu)
  - else call `KeyBindings::rebind`; on `Ok`, clear `RebindRequest` and `RebindError`; on `Err(other_action)`, set `RebindError` with a message naming `other_action` and clear `RebindRequest`
- [x] 3.4 Ensure the `Escape`-cancels-rebind system runs before the open/close-menu `Escape` handler from 2.4 so a cancel doesn't also close the menu (system ordering or an early-return guard)

## 4. Settings menu UI

- [x] 4.1 Add `src/menu/mod.rs` with a `MenuRoot` marker component and a `build_menu` function that spawns the `bevy_ui` `Node` tree: a title, one row per `InputAction` showing the action name and `KeyBindings::bound_key(action)`'s display name, and an error line bound to `RebindError`
- [x] 4.2 Spawn the menu tree in `OnEnter(AppState::Menu)`, despawn the `MenuRoot` entity (and children) in `OnExit(AppState::Menu)`
- [x] 4.3 Add click observers/interaction handling on each binding row: on click, set `RebindRequest(Some(action))`
- [x] 4.4 Add a system (runs in `AppState::Menu`) that updates each row's displayed key text from `KeyBindings` and shows "press a key…" on the row currently targeted by `RebindRequest`
- [x] 4.5 Add a system that updates the error line's text/visibility from `RebindError`
- [x] 4.6 Add a `KeyCode -> display name` helper (e.g. `Space` -> "Space", `ArrowLeft` -> "Left", `KeyW` -> "W") used by both the row text (4.4) and the collision message (3.3/4.5)

## 5. Wiring and verification

- [x] 5.1 Register all new systems/resources/state in `src/main.rs`, keeping `mod input;`, `mod menu;`, `mod app_state;` declarations alongside the existing `mod` list
- [x] 5.2 Run `cargo build` (native) and confirm it compiles without warnings from the new modules
- [x] 5.3 Run `trunk serve` (or `cargo run`) and manually verify: Escape opens the menu, player stops falling/responding to input while open, each row shows the correct default key, clicking a row + pressing a free key rebinds it and gameplay honors the new key after closing, clicking a row + pressing an in-use key shows the rejection message and leaves bindings unchanged, Escape during "awaiting key" cancels without closing the menu, Escape from the menu (not awaiting a key) closes it and resumes gameplay
- [x] 5.4 Run `cargo clippy` and fix any new warnings (repo denies `clippy::shadow_reuse`/`shadow_same`/`shadow_unrelated`)
