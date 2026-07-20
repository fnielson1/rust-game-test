## ADDED Requirements

### Requirement: Opening and closing the menu
The system SHALL let the player open the settings menu with a dedicated key while playing, and close it to return to the same gameplay state.

#### Scenario: Open from gameplay
- **WHEN** the player is playing (menu closed) and presses `Escape`
- **THEN** the settings menu opens and is displayed on screen

#### Scenario: Close back to gameplay
- **WHEN** the settings menu is open (and not currently awaiting a key press for a rebind) and the player presses `Escape`
- **THEN** the settings menu closes and gameplay resumes

### Requirement: Gameplay pauses while the menu is open
The system SHALL stop advancing gameplay simulation while the settings menu is open, and resume it when the menu closes.

#### Scenario: Physics and input freeze while menu is open
- **WHEN** the settings menu is open
- **THEN** the player entity's simulation does not advance (no physics stepping) and gameplay key presses (jump/rotate) have no gameplay effect

#### Scenario: Simulation resumes on close
- **WHEN** the settings menu closes
- **THEN** physics stepping and gameplay input handling resume on the next frame

### Requirement: Menu displays current bindings
While open, the settings menu SHALL display every input action alongside the key currently bound to it.

#### Scenario: Bindings list reflects current state
- **WHEN** the settings menu is open
- **THEN** it shows one row per action (`Jump`, `RotateLeft`, `RotateRight`) with the currently bound key's name next to each

### Requirement: Rebinding an action from the menu
The system SHALL let the player select an action's row in the menu and press a key to rebind that action to the newly pressed key.

#### Scenario: Select a row and press a new key
- **WHEN** the player clicks the `Jump` row and then presses `KeyW`
- **THEN** the menu enters a state indicating it is waiting for a key, and once `KeyW` is pressed the `Jump` row updates to show `KeyW` as its bound key

#### Scenario: Cancel an in-progress rebind
- **WHEN** the player has clicked a row to begin rebinding and then presses `Escape` before pressing another key
- **THEN** the rebind is cancelled, the row's displayed key is unchanged, and the menu remains open

### Requirement: Rejected rebind is shown to the player
When a rebind attempt is rejected due to a key collision, the menu SHALL display a message indicating the rebind was rejected and why.

#### Scenario: Collision message shown
- **WHEN** the player attempts to rebind `Jump` to a key already bound to `RotateLeft`
- **THEN** the menu displays a message that the key is already in use by `RotateLeft`, and `Jump`'s displayed binding is unchanged
