// Pause menu system

use bevy::app::AppExit;
use bevy::prelude::*;

use super::components::{MenuUI, PauseMenuUI};
use crate::core::AppState;

// Type alias for complex button interaction query
type ButtonInteractionQuery<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static Children), (Changed<Interaction>, With<Button>)>;

/// Setup pause menu
pub fn setup_pause_menu(mut commands: Commands, fonts: Res<crate::core::GameFonts>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            MenuUI,
            PauseMenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font: fonts.title.clone(),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(25.0)),
                    ..default()
                },
            ));

            // Resume button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Resume"),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Exit to Menu button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Exit to Menu"),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Exit to Desktop button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Exit"),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// Handle ESC key to toggle pause menu
pub fn handle_pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            AppState::InGame => {
                next_state.set(AppState::PauseMenu);
            }
            AppState::PauseMenu => {
                next_state.set(AppState::InGame);
            }
            _ => {}
        }
    }
}

/// Handle pause menu interactions
pub fn handle_pause_menu(
    interaction_query: ButtonInteractionQuery,
    text_query: Query<&Text>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, children) in &interaction_query {
        if *interaction == Interaction::Pressed {
            if let Some(&child) = children.first() {
                if let Ok(text) = text_query.get(child) {
                    let button_text = text.0.as_str();
                    if button_text.contains("Resume") {
                        next_state.set(AppState::InGame);
                    } else if button_text.contains("Exit to Menu") {
                        next_state.set(AppState::StartMenu);
                    } else if button_text == "Exit" {
                        exit.write(AppExit::Success);
                    }
                }
            }
        }
    }
}
