// Marker components for menu UI entities

use bevy::prelude::*;

/// Marker for any UI entity that should be cleaned up between states
#[derive(Component)]
pub struct MenuUI;

/// Marker for start menu entities
#[derive(Component)]
pub struct StartMenuUI;

/// Marker for player selection menu entities
#[derive(Component)]
pub struct PlayerSelectionUI;

/// Marker for countdown UI entities
#[derive(Component)]
#[allow(dead_code)]
pub struct CountdownUI;

/// Marker for pause menu entities
#[derive(Component)]
pub struct PauseMenuUI;

/// Marker for selected button
#[derive(Component)]
#[allow(dead_code)]
pub struct SelectedButton(pub usize);

/// Marker for turn notification UI
#[derive(Component)]
pub struct TurnNotificationUI;

/// Marker for Player 1 toggle
#[derive(Component)]
pub struct Player1Toggle;

/// Marker for Player 2 toggle
#[derive(Component)]
pub struct Player2Toggle;

/// Marker for Player 1 color button
#[derive(Component)]
pub struct Player1ColorButton {
    pub color_index: usize,
}

/// Marker for Player 2 color button
#[derive(Component)]
pub struct Player2ColorButton {
    pub color_index: usize,
}
