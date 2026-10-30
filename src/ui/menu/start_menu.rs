// Start menu setup and interaction handling

use bevy::app::AppExit;
use bevy::prelude::*;

use super::components::{MenuUI, StartMenuUI};
use crate::core::AppState;

// Type aliases for complex query types
type ButtonInteractionQuery<'w, 's> =
    Query<'w, 's, (&'static Interaction, &'static Children), (Changed<Interaction>, With<Button>)>;
type ButtonColorQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (
        Changed<Interaction>,
        With<Button>,
        Without<crate::ui::menu::components::Player1ColorButton>,
        Without<crate::ui::menu::components::Player2ColorButton>,
    ),
>;

/// Setup the start menu
pub fn setup_start_menu(mut commands: Commands, fonts: Res<crate::core::GameFonts>) {
    // UI overlay - centered
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
            MenuUI,
            StartMenuUI,
        ))
        .with_children(|parent| {
            // Semi-transparent background panel
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(30.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|parent| {
                    // Title
                    parent.spawn((
                        Text::new("BLOCCE"),
                        TextFont {
                            font: fonts.title.clone(),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    // Start button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(35.0),
                                margin: UiRect::all(Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Start Game"),
                                TextFont {
                                    font: fonts.ui.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Exit button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(35.0),
                                margin: UiRect::all(Val::Px(6.0)),
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
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

/// Handle start menu interactions
pub fn handle_start_menu(
    interaction_query: ButtonInteractionQuery,
    text_query: Query<&Text>,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, children) in &interaction_query {
        if *interaction == Interaction::Pressed {
            // Get button text to determine which button was clicked
            if let Some(&child) = children.first() {
                if let Ok(text) = text_query.get(child) {
                    let button_text = text.0.as_str();
                    if button_text.contains("Start") {
                        next_state.set(AppState::PlayerSelection);
                    } else if button_text.contains("Exit") {
                        exit.write(AppExit::Success);
                    }
                }
            }
        }
    }
}

/// Update button colors on hover
pub fn update_button_colors(mut query: ButtonColorQuery) {
    for (interaction, mut bg_color) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = Color::srgba(1.0, 1.0, 1.0, 0.1).into();
            }
            Interaction::Hovered => {
                *bg_color = Color::srgba(1.0, 1.0, 1.0, 0.05).into();
            }
            Interaction::None => {
                *bg_color = Color::NONE.into();
            }
        }
    }
}
