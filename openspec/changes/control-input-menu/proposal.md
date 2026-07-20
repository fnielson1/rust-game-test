## Why

Player controls (`jump`, `rotate left`, `rotate right`) are hardcoded to specific `KeyCode`s inside `player_input.rs`. There's no way for a player to see what the controls are or change them, which is a usability gap as soon as this leaves "scratch" prototyping and gets played by anyone other than the developer.

## What Changes

- Introduce an action-based input mapping layer: gameplay code reads abstract actions (`Jump`, `RotateLeft`, `RotateRight`) from a resource instead of hardcoded `KeyCode`s, and that resource is the single source of truth for the active bindings.
- Add an in-game settings menu (opened via `Escape`) that lists each action and its currently bound key.
- Let the player click a binding, press a new key, and have it take effect immediately for gameplay.
- Pause simulation (physics/gameplay update) while the menu is open; resume on close.
- Guard against binding collisions: attempting to bind a key already in use by another action is rejected with an inline message instead of silently creating a conflict.
- Bindings are in-memory only for this change (reset to defaults on page reload) — persistence (e.g. browser local storage) is called out as a follow-up, not built here.

## Capabilities

### New Capabilities
- `input-bindings`: Resource-backed action→key mapping (defaults, lookup, rebind, collision detection) that gameplay systems query instead of hardcoded `KeyCode`s.
- `settings-menu`: In-game UI (open/close, list bindings, "press a key to rebind" capture flow, pause-while-open behavior) built on Bevy's UI (`bevy_ui`).

### Modified Capabilities
(none — no existing specs in this repo yet)

## Impact

- **Code**: `src/player/player_input.rs` (read from `input-bindings` resource instead of literal `KeyCode`s), `src/main.rs` (register new resource/systems, add a run condition so `player_input`/physics pause while the menu is open), new modules for the bindings resource and the menu UI (e.g. `src/input/` and `src/menu/`).
- **Dependencies**: none new — uses Bevy's built-in `bevy_ui`, no external crate required.
- **Save/version compat**: none (game has no existing save format).
