// Cleanup systems for menu UI

use bevy::prelude::*;

use super::components::MenuUI;
use crate::core::{GameConfiguration, GameState};

/// Cleanup menu UI when transitioning states
pub fn cleanup_menu_ui(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Reset game state when returning to menu
pub fn reset_game_state(mut game_state: ResMut<GameState>, config: Res<GameConfiguration>) {
    *game_state = GameState::from_config(&config);
}
