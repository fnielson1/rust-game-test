use crate::input::RebindRequest;
use avian2d::prelude::{Physics, PhysicsTime};
use bevy::prelude::{ButtonInput, KeyCode, NextState, Res, ResMut, State, States, Time};

#[derive(States, Default, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AppState {
  #[default]
  InGame,
  Menu,
}

/// Opens the settings menu and pauses physics to match.
pub fn open_menu(next_state: &mut NextState<AppState>, physics_time: &mut Time<Physics>) {
  next_state.set(AppState::Menu);
  physics_time.pause();
}

/// Closes the settings menu and unpauses physics to match.
pub fn close_menu(next_state: &mut NextState<AppState>, physics_time: &mut Time<Physics>) {
  next_state.set(AppState::InGame);
  physics_time.unpause();
}

/// Toggles between `Playing` and `Menu` on Escape, pausing/unpausing physics to match.
/// While a rebind is in progress (`RebindRequest` is `Some`), Escape is left for the
/// rebind-capture system to consume as a cancel instead of closing the menu.
pub fn toggle_menu(
  keys: Res<ButtonInput<KeyCode>>,
  state: Res<State<AppState>>,
  mut next_state: ResMut<NextState<AppState>>,
  mut physics_time: ResMut<Time<Physics>>,
  rebind_request: Res<RebindRequest>,
) {
  if !keys.just_pressed(KeyCode::Escape) {
    return;
  }
  match state.get() {
    AppState::InGame => open_menu(&mut next_state, &mut physics_time),
    AppState::Menu => {
      if rebind_request.0.is_some() {
        return;
      }
      close_menu(&mut next_state, &mut physics_time);
    }
  }
}
