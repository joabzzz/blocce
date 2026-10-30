// App-level state management for menu navigation
use bevy::prelude::*;

/// Top-level application states for menu navigation
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    /// Loading state - ensures all resources are loaded before showing menu
    #[default]
    Loading,
    StartMenu,
    PlayerSelection,
    Countdown,
    InGame,
    PauseMenu,
}

/// Player type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum PlayerType {
    Human,
    CPU,
}

/// Game setup configuration from menu selections
#[derive(Resource, Debug, Clone)]
pub struct GameSetup {
    pub player1_type: PlayerType,
    pub player2_type: PlayerType,
    pub player1_color_index: usize,
    pub player2_color_index: usize,
}

impl Default for GameSetup {
    fn default() -> Self {
        Self {
            player1_type: PlayerType::Human,
            player2_type: PlayerType::Human,
            player1_color_index: 0, // Red-orange
            player2_color_index: 4, // Cyan (opposite side of color wheel)
        }
    }
}

impl GameSetup {
    /// Generate 9 equidistant hue colors with decent luma and high saturation
    pub fn get_color_palette() -> Vec<Color> {
        let mut colors = Vec::new();
        for i in 0..9 {
            let hue = i as f32 * 40.0; // 360/9 = 40 degrees each
                                       // High saturation (0.9) and decent lightness (0.55) for vibrant colors
                                       // Convert HSL to sRGB explicitly for UI rendering
            let hsl_color = Color::hsl(hue, 0.9, 0.55);
            let srgba = hsl_color.to_srgba();
            colors.push(Color::srgba(
                srgba.red,
                srgba.green,
                srgba.blue,
                srgba.alpha,
            ));
        }
        colors
    }

    /// Get emissive color from base color (brighter version)
    pub fn get_emissive_from_color(color: Color) -> LinearRgba {
        let linear = color.to_linear();
        LinearRgba::rgb(linear.red * 1.75, linear.green * 1.75, linear.blue * 1.75)
    }
}

/// Countdown timer resource
#[derive(Resource)]
pub struct CountdownTimer {
    pub remaining: f32,
}

impl CountdownTimer {
    pub fn new() -> Self {
        Self { remaining: 2.0 } // Changed to 2 seconds
    }
}

impl Default for CountdownTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Turn notification timer resource
#[derive(Resource)]
pub struct TurnNotification {
    pub timer: f32,
    pub showing: bool,
}

impl TurnNotification {
    pub fn new() -> Self {
        Self {
            timer: 1.5, // Show for 1.5 seconds
            showing: true,
        }
    }
}

impl Default for TurnNotification {
    fn default() -> Self {
        Self::new()
    }
}
