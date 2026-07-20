# input-bindings Specification

## Purpose
Resource-backed action-to-key mapping (defaults, lookup, rebind, collision detection) that gameplay systems query instead of hardcoded `KeyCode`s.

## Requirements

### Requirement: Default key bindings
The system SHALL provide a default keyboard binding for each of the three input actions (`Jump`, `RotateLeft`, `RotateRight`) available at startup, before any player rebinding occurs.

#### Scenario: Fresh app start has usable defaults
- **WHEN** the app starts and no rebinding has occurred
- **THEN** `Jump` is bound to `Space`, `RotateLeft` is bound to `ArrowLeft`, and `RotateRight` is bound to `ArrowRight`

### Requirement: Gameplay reads bindings, not literal key codes
Gameplay input handling SHALL determine which action was triggered by looking up the currently bound key for each action, rather than checking a hardcoded `KeyCode` value.

#### Scenario: Rotate action follows a changed binding
- **WHEN** `RotateLeft` has been rebound from `ArrowLeft` to `KeyA`, and the player presses `KeyA`
- **THEN** the player rotates left, and pressing the old `ArrowLeft` key no longer rotates left

#### Scenario: Jump action follows a changed binding
- **WHEN** `Jump` has been rebound from `Space` to `KeyW`, and the player presses `KeyW`
- **THEN** the player jumps, and pressing `Space` no longer jumps

### Requirement: Rebinding an action
The system SHALL allow the current key bound to a given action to be replaced with a different key at runtime.

#### Scenario: Successful rebind
- **WHEN** a rebind is requested for `Jump` and the next key pressed is `KeyW`, which is not bound to any other action
- **THEN** `Jump` becomes bound to `KeyW` and this takes effect for subsequent input immediately

### Requirement: Duplicate binding rejection
The system SHALL reject a rebind attempt that would bind an action to a key already bound to a different action, leaving both actions' existing bindings unchanged.

#### Scenario: Attempted collision is rejected
- **WHEN** `RotateLeft` is bound to `ArrowLeft`, and a rebind is requested for `Jump` where the next key pressed is `ArrowLeft`
- **THEN** `Jump` remains bound to its previous key, `RotateLeft` remains bound to `ArrowLeft`, and the rebind attempt is reported as rejected

#### Scenario: Rebinding an action to its own current key is not a collision
- **WHEN** a rebind is requested for `Jump`, currently bound to `Space`, and the next key pressed is `Space`
- **THEN** the rebind succeeds (no-op change), and it is not reported as rejected
