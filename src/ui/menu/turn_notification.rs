// Turn notification system

use bevy::prelude::*;

use super::components::TurnNotificationUI;
use crate::core::GameState;

/// Detect player change and spawn turn notification
pub fn show_turn_notification(
    mut commands: Commands,
    game_state: Res<GameState>,
    existing_notification: Query<Entity, With<TurnNotificationUI>>,
    mut local_previous_player: Local<Option<usize>>,
    fonts: Res<crate::core::GameFonts>,
) {
    // Skip if we're not in Playing phase
    if game_state.phase != crate::core::GamePhase::Playing {
        // Reset tracking when not playing
        *local_previous_player = None;
        return;
    }

    // Check if there's already a notification showing
    if !existing_notification.is_empty() {
        return;
    }

    // Check if player actually changed
    let current = game_state.current_player;
    if let Some(previous) = *local_previous_player {
        if previous != current {
            // Player changed! Show notification
            commands.insert_resource(crate::core::TurnNotification::new());

            // Spawn turn notification UI
            commands
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    TurnNotificationUI,
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
                                Text::new(format!("Player {}'s Turn", current + 1)),
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
    }

    // Update tracking
    *local_previous_player = Some(current);
}

/// Update turn notification timer and despawn when done
pub fn update_turn_notification(
    mut commands: Commands,
    turn_notification: Option<ResMut<crate::core::TurnNotification>>,
    time: Res<Time>,
    notification_ui: Query<Entity, With<TurnNotificationUI>>,
) {
    let Some(mut notif) = turn_notification else {
        return;
    };

    if !notif.showing {
        return;
    }

    notif.timer -= time.delta_secs();

    if notif.timer <= 0.0 {
        notif.showing = false;

        // Despawn the notification UI
        for entity in notification_ui.iter() {
            commands.entity(entity).despawn();
        }

        // Remove the resource
        commands.remove_resource::<crate::core::TurnNotification>();
    }
}
