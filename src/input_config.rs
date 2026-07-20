use bevy::prelude::{ButtonInput, KeyCode, Res, ResMut, Resource};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InputAction {
  Jump,
  RotateLeft,
  RotateRight,
}

impl InputAction {
  pub const ALL: [InputAction; 3] = [
    InputAction::Jump,
    InputAction::RotateLeft,
    InputAction::RotateRight,
  ];

  pub fn label(self) -> &'static str {
    match self {
      InputAction::Jump => "Jump",
      InputAction::RotateLeft => "Rotate Left",
      InputAction::RotateRight => "Rotate Right",
    }
  }
}

#[derive(Resource)]
pub struct KeyBindings(HashMap<InputAction, KeyCode>);

impl Default for KeyBindings {
  fn default() -> Self {
    let mut bindings = HashMap::new();
    bindings.insert(InputAction::Jump, KeyCode::KeyW);
    bindings.insert(InputAction::RotateLeft, KeyCode::KeyA);
    bindings.insert(InputAction::RotateRight, KeyCode::KeyD);
    Self(bindings)
  }
}

impl KeyBindings {
  pub fn bound_key(&self, action: InputAction) -> KeyCode {
    self.0[&action]
  }

  pub fn action_for_key(&self, key: KeyCode) -> Option<InputAction> {
    self
      .0
      .iter()
      .find(|&(_, &bound_key)| bound_key == key)
      .map(|(&action, _)| action)
  }

  /// Rebinds `action` to `key`. Rejects the change (returning the conflicting action)
  /// if `key` is already bound to a *different* action.
  pub fn rebind(&mut self, action: InputAction, key: KeyCode) -> Result<(), InputAction> {
    if let Some(existing_action) = self.action_for_key(key)
      && existing_action != action
    {
      return Err(existing_action);
    }
    self.0.insert(action, key);
    Ok(())
  }
}

/// Which action is currently waiting for the next key press to rebind to, if any.
#[derive(Resource, Default)]
pub struct RebindRequest(pub Option<InputAction>);

/// Message describing why the last rebind attempt was rejected, if any.
#[derive(Resource, Default)]
pub struct RebindError(pub Option<String>);

/// While `RebindRequest` targets an action, consumes the next key press: `Escape` cancels
/// the rebind, a free key rebinds the action, an already-bound key is rejected with a
/// message left in `RebindError`.
pub fn rebind_capture(
  keys: Res<ButtonInput<KeyCode>>,
  mut request: ResMut<RebindRequest>,
  mut bindings: ResMut<KeyBindings>,
  mut error: ResMut<RebindError>,
) {
  let Some(action) = request.0 else {
    return;
  };
  let Some(&pressed_key) = keys.get_just_pressed().next() else {
    return;
  };
  if pressed_key == KeyCode::Escape {
    request.0 = None;
    return;
  }
  match bindings.rebind(action, pressed_key) {
    Ok(()) => {
      request.0 = None;
      error.0 = None;
    }
    Err(other_action) => {
      request.0 = None;
      error.0 = Some(format!(
        "{} is already bound to {}",
        key_display_name(pressed_key),
        other_action.label()
      ));
    }
  }
}

/// Human-readable name for a `KeyCode`, used by the settings menu and rebind-rejection messages.
pub fn key_display_name(key: KeyCode) -> String {
  match key {
    KeyCode::Space => "Space".to_string(),
    KeyCode::Escape => "Escape".to_string(),
    KeyCode::ArrowUp => "Up".to_string(),
    KeyCode::ArrowDown => "Down".to_string(),
    KeyCode::ArrowLeft => "Left".to_string(),
    KeyCode::ArrowRight => "Right".to_string(),
    other => format!("{other:?}"),
  }
}
