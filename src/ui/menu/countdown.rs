// Countdown system before game starts

use bevy::prelude::*;

use super::components::{CountdownUI, MenuUI};
use crate::core::{AppState, CountdownTimer, GameSetup, PlayerManager};

/// Setup countdown - 2 second delay with "Starting game..." text
pub fn setup_countdown(
    mut commands: Commands,
    fonts: Res<crate::core::GameFonts>,
    game_setup: Res<GameSetup>,
) {
    commands.insert_resource(CountdownTimer::new());

    // Update PlayerManager with selected colors from GameSetup
    let player_manager = PlayerManager::from_game_setup(&game_setup);
    commands.insert_resource(player_manager);

    // Small "Starting game..." overlay
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
            CountdownUI,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(15.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Starting game..."),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// Update countdown timer - silent countdown
pub fn update_countdown(
    mut timer: ResMut<CountdownTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    timer.remaining -= time.delta_secs();

    // Transition to game when countdown finishes
    if timer.remaining <= 0.0 {
        next_state.set(AppState::InGame);
    }
}
