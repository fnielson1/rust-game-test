use crate::app_state::{close_menu, open_menu, AppState};
use crate::input::{key_display_name, InputAction, KeyBindings, RebindError, RebindRequest};
use avian2d::prelude::Physics;
use bevy::prelude::{
  default, AlignItems, BackgroundColor, BorderRadius, Changed, Color, Commands, Component,
  FlexDirection, Interaction, JustifyContent, NextState, Node, PositionType, Query, Res, ResMut,
  State, Text, TextColor, Time, UiRect, UiTransform, Val, With,
};
use bevy::math::Rot2;
use bevy::state::state_scoped::DespawnOnExit;

const PANEL_BACKGROUND: Color = Color::srgba(0.1, 0.1, 0.12, 0.92);
const ROW_BACKGROUND: Color = Color::srgba(0.2, 0.2, 0.24, 1.0);
const ROW_AWAITING_BACKGROUND: Color = Color::srgba(0.35, 0.3, 0.1, 1.0);
const ERROR_TEXT_COLOR: Color = Color::srgb(0.9, 0.3, 0.3);

const COG_BUTTON_SIZE: f32 = 40.0;
const COG_BUTTON_BACKGROUND: Color = Color::srgba(0.15, 0.15, 0.18, 0.85);
const COG_TOOTH_SIZE: f32 = 22.0;
const COG_HUB_SIZE: f32 = 12.0;
const COG_ICON_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);

/// Marks the always-present top-left button that opens the settings menu.
#[derive(Component)]
pub struct CogButton;

/// Spawns the cog icon button once at startup; it stays present across `AppState` changes.
pub fn spawn_cog_button(mut commands: Commands) {
  commands
    .spawn((
      Node {
        position_type: PositionType::Absolute,
        top: Val::Px(12.0),
        left: Val::Px(12.0),
        width: Val::Px(COG_BUTTON_SIZE),
        height: Val::Px(COG_BUTTON_SIZE),
        border_radius: BorderRadius::all(Val::Percent(50.0)),
        ..default()
      },
      BackgroundColor(COG_BUTTON_BACKGROUND),
      Interaction::None,
      CogButton,
    ))
    .with_children(|button| {
      // A gear glyph built from two overlapping squares (an 8-point star) plus a hub dot,
      // since the bundled font subset has no cog/settings symbol to draw as text.
      for rotation_degrees in [0.0, 45.0] {
        button.spawn((
          Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(50.0),
            width: Val::Px(COG_TOOTH_SIZE),
            height: Val::Px(COG_TOOTH_SIZE),
            margin: UiRect {
              left: Val::Px(-COG_TOOTH_SIZE / 2.0),
              top: Val::Px(-COG_TOOTH_SIZE / 2.0),
              ..default()
            },
            ..default()
          },
          UiTransform::from_rotation(Rot2::degrees(rotation_degrees)),
          BackgroundColor(COG_ICON_COLOR),
        ));
      }
      button.spawn((
        Node {
          position_type: PositionType::Absolute,
          top: Val::Percent(50.0),
          left: Val::Percent(50.0),
          width: Val::Px(COG_HUB_SIZE),
          height: Val::Px(COG_HUB_SIZE),
          margin: UiRect {
            left: Val::Px(-COG_HUB_SIZE / 2.0),
            top: Val::Px(-COG_HUB_SIZE / 2.0),
            ..default()
          },
          border_radius: BorderRadius::all(Val::Percent(50.0)),
          ..default()
        },
        BackgroundColor(COG_BUTTON_BACKGROUND),
      ));
    });
}

/// Toggles the settings menu open/closed when the cog button is clicked. While a rebind is
/// in progress, the click is ignored so it doesn't abandon the pending capture mid-flight.
pub fn handle_cog_click(
  cog: Query<&Interaction, (Changed<Interaction>, With<CogButton>)>,
  state: Res<State<AppState>>,
  mut next_state: ResMut<NextState<AppState>>,
  mut physics_time: ResMut<Time<Physics>>,
  rebind_request: Res<RebindRequest>,
) {
  for interaction in &cog {
    if *interaction != Interaction::Pressed {
      continue;
    }
    match state.get() {
      AppState::InGame => open_menu(&mut next_state, &mut physics_time),
      AppState::Menu => {
        if rebind_request.0.is_some() {
          continue;
        }
        close_menu(&mut next_state, &mut physics_time);
      }
    }
  }
}

/// Marks the row's key-label text with the action it displays.
#[derive(Component)]
pub struct BindingKeyLabel(pub InputAction);

/// Marks a clickable binding row with the action it rebinds.
#[derive(Component)]
pub struct BindingRow(pub InputAction);

/// Marks the text node that shows the last rebind-rejection message.
#[derive(Component)]
pub struct ErrorLabel;

pub fn spawn_menu(mut commands: Commands, bindings: Res<KeyBindings>) {
  commands
    .spawn((
      Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
      },
      DespawnOnExit(AppState::Menu),
    ))
    .with_children(|screen| {
      screen
        .spawn((
          Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(24.0)),
            row_gap: Val::Px(12.0),
            min_width: Val::Px(320.0),
            ..default()
          },
          BackgroundColor(PANEL_BACKGROUND),
        ))
        .with_children(|panel| {
          panel.spawn((Text::new("Controls"), TextColor(Color::WHITE)));

          for action in InputAction::ALL {
            panel
              .spawn((
                Node {
                  flex_direction: FlexDirection::Row,
                  justify_content: JustifyContent::SpaceBetween,
                  padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                  ..default()
                },
                BackgroundColor(ROW_BACKGROUND),
                Interaction::None,
                BindingRow(action),
              ))
              .with_children(|row| {
                row.spawn((Text::new(action.label()), TextColor(Color::WHITE)));
                row.spawn((
                  Text::new(key_display_name(bindings.bound_key(action))),
                  TextColor(Color::WHITE),
                  BindingKeyLabel(action),
                ));
              });
          }

          panel.spawn((Text::new(""), TextColor(ERROR_TEXT_COLOR), ErrorLabel));
        });
    });
}

/// Handles clicks on binding rows by requesting a rebind for that row's action.
pub fn handle_row_clicks(
  rows: Query<(&Interaction, &BindingRow), Changed<Interaction>>,
  mut request: ResMut<RebindRequest>,
  mut error: ResMut<RebindError>,
) {
  for (interaction, row) in &rows {
    if *interaction == Interaction::Pressed {
      request.0 = Some(row.0);
      error.0 = None;
    }
  }
}

/// Keeps each row's displayed key and highlight in sync with `KeyBindings`/`RebindRequest`.
pub fn update_binding_rows(
  bindings: Res<KeyBindings>,
  request: Res<RebindRequest>,
  mut labels: Query<(&BindingKeyLabel, &mut Text)>,
  mut rows: Query<(&BindingRow, &mut BackgroundColor)>,
) {
  for (label, mut text) in &mut labels {
    let awaiting = request.0 == Some(label.0);
    text.0 = if awaiting {
      "press a key…".to_string()
    } else {
      key_display_name(bindings.bound_key(label.0))
    };
  }
  for (row, mut background) in &mut rows {
    let awaiting = request.0 == Some(row.0);
    *background = BackgroundColor(if awaiting {
      ROW_AWAITING_BACKGROUND
    } else {
      ROW_BACKGROUND
    });
  }
}

/// Keeps the error line in sync with the last rebind rejection, if any.
pub fn update_error_label(error: Res<RebindError>, mut labels: Query<&mut Text, With<ErrorLabel>>) {
  let Ok(mut text) = labels.single_mut() else {
    return;
  };
  text.0 = error.0.clone().unwrap_or_default();
}
