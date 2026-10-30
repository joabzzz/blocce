// Dynamic lighting system for blocks
use bevy::prelude::*;

use crate::core::{Block, GameConfiguration, PlayerBlock, PlayerManager};

/// Marker component for block lights
#[derive(Component)]
pub struct BlockLight;

/// Attach point lights to blocks when they're spawned
#[allow(clippy::type_complexity)]
pub fn attach_lights_to_blocks(
    mut commands: Commands,
    blocks_without_lights: Query<(Entity, &PlayerBlock), (With<Block>, Without<BlockLight>)>,
    player_manager: Res<PlayerManager>,
    config: Res<GameConfiguration>,
) {
    let light_config = &config.lighting.block_light;

    for (entity, player_block) in blocks_without_lights.iter() {
        let player_config = player_manager.get_player(player_block.player).unwrap();

        // Create a point light that matches the block's color
        let light_color = player_config.color;

        commands.entity(entity).insert(BlockLight);

        // Spawn a child entity with the point light
        commands.entity(entity).with_children(|parent| {
            parent.spawn(PointLight {
                color: light_color,
                intensity: light_config.intensity,
                radius: light_config.radius,
                range: light_config.range,
                shadows_enabled: light_config.shadows_enabled,
                ..default()
            });
        });
    }
}

/// Update light intensity during victory animations and fade-outs
pub fn update_block_lights(
    block_query: Query<(&Children, &PlayerBlock), With<Block>>,
    mut light_query: Query<&mut PointLight>,
    player_manager: Res<PlayerManager>,
    config: Res<GameConfiguration>,
) {
    let light_config = &config.lighting.block_light;

    for (children, player_block) in block_query.iter() {
        // Find the light child
        for child in children.iter() {
            if let Ok(mut point_light) = light_query.get_mut(child) {
                let player_config = player_manager.get_player(player_block.player).unwrap();

                // Keep the light color matched to player color
                point_light.color = player_config.color;

                // Maintain configured intensity
                point_light.intensity = light_config.intensity;
            }
        }
    }
}
