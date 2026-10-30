use bevy::prelude::*;

/// Resource containing handles to all custom fonts used in the game
#[derive(Resource)]
pub struct GameFonts {
    /// Silkscreen font - used for the main title and retro-style headers
    pub title: Handle<Font>,
    /// Inter font - used for general UI text, buttons, and body text
    pub ui: Handle<Font>,
}

impl GameFonts {
    /// Load all game fonts from the asset server
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            title: asset_server.load("Silkscreen-Regular.ttf"),
            ui: asset_server.load("Inter/Inter-VariableFont_opsz,wght.ttf"),
        }
    }
}
