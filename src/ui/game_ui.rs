// UI setup and update systems
use bevy::prelude::*;

use crate::core::{
    GameConfiguration, GamePhase, GameSetup, GameText, Player1ScoreText, Player2ScoreText,
    PlayerManager, PlayerType,
};

#[derive(Component)]
pub struct GameUiRoot;

#[derive(Component)]
pub struct ControlsText;

/// Setup the game UI
pub fn setup_ui(
    mut commands: Commands,
    player_manager: Res<PlayerManager>,
    config: Res<GameConfiguration>,
    fonts: Res<crate::core::GameFonts>,
) {
    let player1 = player_manager.get_player(0).unwrap();
    let player2 = player_manager.get_player(1).unwrap();
    let ui_cfg = &config.ui;

    // UI root
    commands
        .spawn((GameUiRoot, Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(ui_cfg.score_margin)),
            ..default()
        }))
        .with_children(|parent| {
            // Top row with player scores
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|parent| {
                    // Player 1 score (top left)
                    parent.spawn((
                        Text::new(format!("{} - Score: 0", player1.name)),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: ui_cfg.score_text.size,
                            ..default()
                        },
                        TextColor(player1.color),
                        Player1ScoreText,
                    ));

                    // Center game text
                    parent.spawn((
                        Text::new(format!(
                            "{}'s Turn - Blocks: 0/{}\nP1: -- | P2: --",
                            player1.name, config.game.blocks_per_player
                        )),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: ui_cfg.game_text.size.max(16.0),
                            ..default()
                        },
                        TextColor(ui_cfg.game_text.color()),
                        GameText,
                    ));

                    // Player 2 score (top right)
                    parent.spawn((
                        Text::new(format!("{} - Score: 0", player2.name)),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: ui_cfg.score_text.size,
                            ..default()
                        },
                        TextColor(player2.color),
                        Player2ScoreText,
                        TextLayout {
                            justify: Justify::Right,
                            ..default()
                        },
                    ));
                });

            // A compact, persistent guide stays readable over the arena.
            parent
                .spawn((
                    Node {
                        align_self: AlignSelf::Center,
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Land closest to the small white block to win the round\nA / D  Aim     Hold SPACE  Charge     Release  Throw\nESC  Pause"),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TextLayout { justify: Justify::Center, ..default() },
                        ControlsText,
                    ));
                });
        });
}

/// Update UI text based on game state
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update_ui(
    game_state: Res<crate::core::GameState>,
    player_manager: Res<PlayerManager>,
    config: Res<GameConfiguration>,
    setup: Res<GameSetup>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut controls_query: Query<
        &mut Text,
        (
            With<ControlsText>,
            Without<GameText>,
            Without<Player1ScoreText>,
            Without<Player2ScoreText>,
        ),
    >,
    mut game_text_query: Query<(&mut Text, &mut TextFont), With<GameText>>,
    mut player1_score_query: Query<
        &mut Text,
        (
            With<Player1ScoreText>,
            Without<GameText>,
            Without<Player2ScoreText>,
        ),
    >,
    mut player2_score_query: Query<
        &mut Text,
        (
            With<Player2ScoreText>,
            Without<GameText>,
            Without<Player1ScoreText>,
        ),
    >,
) {
    let player1 = player_manager.get_player(0).unwrap();
    let player2 = player_manager.get_player(1).unwrap();
    let blocks_per_player = config.game.blocks_per_player;

    // Update game state text
    if let Ok((mut text, mut font)) = game_text_query.single_mut() {
        let current_player = player_manager
            .get_player(game_state.current_player)
            .unwrap();
        let blocks_remaining =
            blocks_per_player - game_state.blocks_thrown[game_state.current_player];

        // Format distance strings
        let dist1_str = if game_state.scores[0] == 0.0 && game_state.blocks_thrown[0] == 0 {
            "--".to_string()
        } else if game_state.scores[0] >= f32::MAX - 1.0 {
            "OFF".to_string()
        } else {
            format!("{:.2}", game_state.scores[0])
        };

        let dist2_str = if game_state.scores[1] == 0.0 && game_state.blocks_thrown[1] == 0 {
            "--".to_string()
        } else if game_state.scores[1] >= f32::MAX - 1.0 {
            "OFF".to_string()
        } else {
            format!("{:.2}", game_state.scores[1])
        };

        // Display different messages based on game phase
        match game_state.phase {
            GamePhase::Playing => {
                // Use normal font size
                font.font_size = config.ui.game_text.size.max(16.0);
                if !game_state.all_blocks_settled {
                    **text = format!(
                        "{}'s Turn - Waiting for blocks to settle...\nThrown: {}/{}\nClosest distance  P1: {} | P2: {}",
                        current_player.name,
                        blocks_per_player - blocks_remaining,
                        blocks_per_player,
                        dist1_str,
                        dist2_str
                    );
                } else {
                    **text = format!(
                        "{}'s Turn - Thrown: {}/{}\nClosest distance  P1: {} | P2: {}",
                        current_player.name,
                        blocks_per_player - blocks_remaining,
                        blocks_per_player,
                        dist1_str,
                        dist2_str
                    );
                }
            }
            GamePhase::BlocksSettling => {
                // Use normal font size
                font.font_size = config.ui.game_text.size.max(16.0);
                **text = format!(
                    "All Blocks Thrown - Settling...\nClosest distance  P1: {} | P2: {}",
                    dist1_str, dist2_str
                );
            }
            GamePhase::VictoryAnimation | GamePhase::FadingOut => {
                // Use normal font size
                font.font_size = config.ui.game_text.size.max(16.0);
                let winner = if let Some(winner_id) = game_state.round_winner {
                    let winner = player_manager.get_player(winner_id).unwrap();
                    format!("{} Wins Round!", winner.name)
                } else {
                    "Round Tied!".to_string()
                };
                **text = format!(
                    "{}\nClosest distance  P1: {} | P2: {}",
                    winner, dist1_str, dist2_str
                );
            }
            GamePhase::MatchComplete => {
                // Use larger font for match winner
                font.font_size = config.ui.match_winner_text.size;

                let winner = if game_state.round_wins[0] > game_state.round_wins[1] {
                    format!("{} WINS THE MATCH!", player1.name)
                } else {
                    format!("{} WINS THE MATCH!", player2.name)
                };
                **text = format!(
                    "{}\n\nFinal rounds: {} - {}\n\nPress SPACE to play again",
                    winner, game_state.round_wins[0], game_state.round_wins[1]
                );
            }
        }
    }

    if let Ok(mut text) = controls_query.single_mut() {
        let cpu_turn = match game_state.current_player {
            0 => setup.player1_type == PlayerType::CPU,
            _ => setup.player2_type == PlayerType::CPU,
        };
        **text = if game_state.phase == GamePhase::MatchComplete {
            "SPACE  Play again     ESC  Pause".to_string()
        } else if game_state.phase != GamePhase::Playing || !game_state.all_blocks_settled {
            "Closest to the white block wins     •     ESC  Pause".to_string()
        } else if cpu_turn {
            "CPU is aiming...     •     ESC  Pause".to_string()
        } else {
            let power = (game_state.throw_power * 100.0).round();
            let throw_hint = if keyboard.pressed(KeyCode::Space) {
                "Release SPACE to throw"
            } else {
                "Hold SPACE to charge, release to throw"
            };
            format!("Land closest to the small white block • First to {} rounds wins\nA / D  Aim     •     {}     •     Power: {:.0}%\nUp / Down / W / S  Camera     •     ESC  Pause", config.game.rounds_to_win, throw_hint, power)
        };
    }

    // Update Player 1 score (show round wins)
    if let Ok(mut text) = player1_score_query.single_mut() {
        **text = format!("{}  |  Rounds: {}", player1.name, game_state.round_wins[0]);
    }

    // Update Player 2 score (show round wins)
    if let Ok(mut text) = player2_score_query.single_mut() {
        **text = format!("{}  |  Rounds: {}", player2.name, game_state.round_wins[1]);
    }
}
