// Player configuration and management
use super::config::{GameConfiguration, PlayerConfigData};
use bevy::prelude::*;

/// Configuration for a player including visual appearance
#[derive(Clone, Debug)]
pub struct PlayerConfig {
    pub name: String,
    pub color: Color,         // Base color for blocks and UI
    pub emissive: LinearRgba, // Emissive glow for blocks
}

impl PlayerConfig {
    /// Create from config data
    pub fn from_config_data(data: &PlayerConfigData) -> Self {
        Self {
            name: data.name.clone(),
            color: data.color.to_srgb(),
            emissive: data.emissive.to_linear(),
        }
    }
}

/// Resource to manage players in the game
#[derive(Resource)]
pub struct PlayerManager {
    pub players: Vec<PlayerConfig>,
}

impl PlayerManager {
    /// Create from game configuration
    pub fn from_config(config: &GameConfiguration) -> Self {
        Self {
            players: config
                .players
                .iter()
                .map(PlayerConfig::from_config_data)
                .collect(),
        }
    }

    /// Create from GameSetup with custom colors
    pub fn from_game_setup(game_setup: &crate::core::GameSetup) -> Self {
        let color_palette = crate::core::GameSetup::get_color_palette();
        let p1_color = color_palette[game_setup.player1_color_index];
        let p2_color = color_palette[game_setup.player2_color_index];

        Self {
            players: vec![
                PlayerConfig {
                    name: "Player 1".to_string(),
                    color: p1_color,
                    emissive: crate::core::GameSetup::get_emissive_from_color(p1_color),
                },
                PlayerConfig {
                    name: "Player 2".to_string(),
                    color: p2_color,
                    emissive: crate::core::GameSetup::get_emissive_from_color(p2_color),
                },
            ],
        }
    }

    /// Get configuration for a specific player
    pub fn get_player(&self, id: usize) -> Option<&PlayerConfig> {
        self.players.get(id)
    }
}
