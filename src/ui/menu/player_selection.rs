// Player selection menu

use bevy::prelude::*;

use super::components::{
    MenuUI, Player1ColorButton, Player1Toggle, Player2ColorButton, Player2Toggle, PlayerSelectionUI,
};
use crate::core::{AppState, GameSetup, PlayerType};

// Type alias for complex player selection query
type PlayerSelectionQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        Option<&'static Children>,
        Option<&'static Player1Toggle>,
        Option<&'static Player2Toggle>,
        Option<&'static Player1ColorButton>,
        Option<&'static Player2ColorButton>,
    ),
    (Changed<Interaction>, With<Button>),
>;

// Type alias for player 1 color button query
type Player1ColorButtonQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Player1ColorButton, &'static mut Node),
    (With<Button>, Without<Player2ColorButton>),
>;

// Type alias for player 2 color button query
type Player2ColorButtonQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Player2ColorButton, &'static mut Node),
    (With<Button>, Without<Player1ColorButton>),
>;

/// Setup player selection menu
pub fn setup_player_selection(
    mut commands: Commands,
    game_setup: Res<GameSetup>,
    fonts: Res<crate::core::GameFonts>,
) {
    let color_palette = GameSetup::get_color_palette();
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            MenuUI,
            PlayerSelectionUI,
        ))
        .with_children(|parent| {
            // Semi-transparent panel
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(25.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|parent| {
                    // Title
                    parent.spawn((
                        Text::new("Player Setup"),
                        TextFont {
                            font: fonts.title.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    // Players row
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(25.0),
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Player 1 column
                            parent
                                .spawn((Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("P1"),
                                        TextFont {
                                            font: fonts.ui.clone(),
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(1.0, 0.5, 0.0)),
                                        Node {
                                            margin: UiRect::bottom(Val::Px(6.0)),
                                            ..default()
                                        },
                                    ));

                                    let p1_text = if game_setup.player1_type == PlayerType::Human {
                                        "Human"
                                    } else {
                                        "CPU"
                                    };

                                    parent
                                        .spawn((
                                            Button,
                                            Player1Toggle,
                                            Node {
                                                width: Val::Px(90.0),
                                                height: Val::Px(32.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::NONE),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new(p1_text),
                                                TextFont {
                                                    font: fonts.ui.clone(),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });

                                    // Player 1 color selection
                                    parent.spawn((
                                        Text::new("Color"),
                                        TextFont {
                                            font: fonts.ui.clone(),
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                        Node {
                                            margin: UiRect {
                                                top: Val::Px(8.0),
                                                bottom: Val::Px(4.0),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                    ));

                                    // Color palette for P1
                                    parent
                                        .spawn((Node {
                                            flex_direction: FlexDirection::Row,
                                            flex_wrap: FlexWrap::Wrap,
                                            justify_content: JustifyContent::Center,
                                            width: Val::Px(90.0),
                                            column_gap: Val::Px(3.0),
                                            row_gap: Val::Px(3.0),
                                            ..default()
                                        },))
                                        .with_children(|parent| {
                                            for (i, color) in color_palette.iter().enumerate() {
                                                let is_selected =
                                                    i == game_setup.player1_color_index;
                                                parent.spawn((
                                                    Button,
                                                    Player1ColorButton { color_index: i },
                                                    Node {
                                                        width: Val::Px(24.0),
                                                        height: Val::Px(24.0),
                                                        border: if is_selected {
                                                            UiRect::all(Val::Px(2.0))
                                                        } else {
                                                            UiRect::all(Val::Px(0.0))
                                                        },
                                                        ..default()
                                                    },
                                                    BackgroundColor(*color),
                                                    BorderColor::all(Color::WHITE),
                                                ));
                                            }
                                        });
                                });

                            // Player 2 column
                            parent
                                .spawn((Node {
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("P2"),
                                        TextFont {
                                            font: fonts.ui.clone(),
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.0, 0.5, 1.0)),
                                        Node {
                                            margin: UiRect::bottom(Val::Px(6.0)),
                                            ..default()
                                        },
                                    ));

                                    let p2_text = if game_setup.player2_type == PlayerType::Human {
                                        "Human"
                                    } else {
                                        "CPU"
                                    };

                                    parent
                                        .spawn((
                                            Button,
                                            Player2Toggle,
                                            Node {
                                                width: Val::Px(90.0),
                                                height: Val::Px(32.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::NONE),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new(p2_text),
                                                TextFont {
                                                    font: fonts.ui.clone(),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });

                                    // Player 2 color selection
                                    parent.spawn((
                                        Text::new("Color"),
                                        TextFont {
                                            font: fonts.ui.clone(),
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                        Node {
                                            margin: UiRect {
                                                top: Val::Px(8.0),
                                                bottom: Val::Px(4.0),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                    ));

                                    // Color palette for P2
                                    parent
                                        .spawn((Node {
                                            flex_direction: FlexDirection::Row,
                                            flex_wrap: FlexWrap::Wrap,
                                            justify_content: JustifyContent::Center,
                                            width: Val::Px(90.0),
                                            column_gap: Val::Px(3.0),
                                            row_gap: Val::Px(3.0),
                                            ..default()
                                        },))
                                        .with_children(|parent| {
                                            for (i, color) in color_palette.iter().enumerate() {
                                                let is_selected =
                                                    i == game_setup.player2_color_index;
                                                parent.spawn((
                                                    Button,
                                                    Player2ColorButton { color_index: i },
                                                    Node {
                                                        width: Val::Px(24.0),
                                                        height: Val::Px(24.0),
                                                        border: if is_selected {
                                                            UiRect::all(Val::Px(2.0))
                                                        } else {
                                                            UiRect::all(Val::Px(0.0))
                                                        },
                                                        ..default()
                                                    },
                                                    BackgroundColor(*color),
                                                    BorderColor::all(Color::WHITE),
                                                ));
                                            }
                                        });
                                });
                        });

                    // Start button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(35.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Start"),
                                TextFont {
                                    font: fonts.ui.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Back button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(35.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Back"),
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

/// Handle player selection interactions
pub fn handle_player_selection(
    interaction_query: PlayerSelectionQuery,
    mut text_writer_query: Query<&mut Text>,
    mut game_setup: ResMut<GameSetup>,
    mut next_state: ResMut<NextState<AppState>>,
    mut all_p1_color_buttons: Player1ColorButtonQuery,
    mut all_p2_color_buttons: Player2ColorButtonQuery,
) {
    for (interaction, children, p1_toggle, p2_toggle, p1_color, p2_color) in &interaction_query {
        if *interaction == Interaction::Pressed {
            // Handle P1 color selection
            if let Some(color_btn) = p1_color {
                game_setup.player1_color_index = color_btn.color_index;
                // Update borders on all P1 color buttons
                for (btn, mut node) in all_p1_color_buttons.iter_mut() {
                    if btn.color_index == color_btn.color_index {
                        node.border = UiRect::all(Val::Px(2.0));
                    } else {
                        node.border = UiRect::all(Val::Px(0.0));
                    }
                }
                continue;
            }
            // Handle P2 color selection
            else if let Some(color_btn) = p2_color {
                game_setup.player2_color_index = color_btn.color_index;
                // Update borders on all P2 color buttons
                for (btn, mut node) in all_p2_color_buttons.iter_mut() {
                    if btn.color_index == color_btn.color_index {
                        node.border = UiRect::all(Val::Px(2.0));
                    } else {
                        node.border = UiRect::all(Val::Px(0.0));
                    }
                }
                continue;
            }
            // Toggle player 1 type
            else if p1_toggle.is_some() {
                game_setup.player1_type = match game_setup.player1_type {
                    PlayerType::Human => PlayerType::CPU,
                    PlayerType::CPU => PlayerType::Human,
                };
                // Update button text directly
                if let Some(children) = children {
                    if let Some(&child) = children.first() {
                        if let Ok(mut text) = text_writer_query.get_mut(child) {
                            text.0 = if game_setup.player1_type == PlayerType::Human {
                                "Human".to_string()
                            } else {
                                "CPU".to_string()
                            };
                        }
                    }
                }
            }
            // Toggle player 2 type
            else if p2_toggle.is_some() {
                game_setup.player2_type = match game_setup.player2_type {
                    PlayerType::Human => PlayerType::CPU,
                    PlayerType::CPU => PlayerType::Human,
                };
                // Update button text directly
                if let Some(children) = children {
                    if let Some(&child) = children.first() {
                        if let Ok(mut text) = text_writer_query.get_mut(child) {
                            text.0 = if game_setup.player2_type == PlayerType::Human {
                                "Human".to_string()
                            } else {
                                "CPU".to_string()
                            };
                        }
                    }
                }
            }
            // Check text for Start/Back buttons
            else if let Some(children) = children {
                if let Some(&child) = children.first() {
                    if let Ok(text) = text_writer_query.get(child) {
                        let button_text = text.0.as_str();
                        if button_text.contains("Start") {
                            next_state.set(AppState::Countdown);
                        } else if button_text.contains("Back") {
                            next_state.set(AppState::StartMenu);
                        }
                    }
                }
            }
        }
    }
}
