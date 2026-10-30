// Victory animations, fade effects, and match completion
use bevy::prelude::*;

use crate::core::{
    Block, FadingIn, FadingOut, GameConfiguration, GamePhase, GameState, MatchVictoryAnimation,
    PlayerBlock, PlayerManager, TargetBlock, VictoryAnimation,
};

/// Start victory animation when entering phase
#[allow(clippy::type_complexity)]
pub fn start_victory_animation(
    mut commands: Commands,
    game_state: Res<GameState>,
    player_blocks: Query<(Entity, &Transform), (With<Block>, Without<VictoryAnimation>)>,
    target_query: Query<(Entity, &Transform), With<TargetBlock>>,
    time: Res<Time>,
) {
    if game_state.phase != GamePhase::VictoryAnimation {
        return;
    }

    let time_in_phase = time.elapsed_secs() - game_state.phase_start_time;
    if time_in_phase > 0.1 {
        return; // Already started
    }

    println!("Starting victory animation...");

    // Find closest block
    if let Ok((target_entity, target_transform)) = target_query.single() {
        let target_pos = target_transform.translation;
        let mut closest_block: Option<(Entity, f32)> = None;

        for (entity, transform) in player_blocks.iter() {
            let distance = transform.translation.distance(target_pos);
            if closest_block.is_none() || distance < closest_block.unwrap().1 {
                closest_block = Some((entity, distance));
            }
        }

        // Mark all blocks with animation
        for (entity, _) in player_blocks.iter() {
            let is_winner = closest_block.is_some_and(|(winner_entity, _)| winner_entity == entity);
            commands.entity(entity).insert(VictoryAnimation {
                start_time: time.elapsed_secs(),
                is_winner,
            });
        }

        // Mark target block with animation too (not a winner, will dim and fade)
        commands.entity(target_entity).insert(VictoryAnimation {
            start_time: time.elapsed_secs(),
            is_winner: false,
        });
    }
}

/// Animate victory - winner supernovas, losers dim
#[allow(clippy::type_complexity)]
pub fn animate_victory(
    animating_blocks: Query<(
        &VictoryAnimation,
        &MeshMaterial3d<StandardMaterial>,
        &PlayerBlock,
    )>,
    animating_target: Query<
        (&VictoryAnimation, &MeshMaterial3d<StandardMaterial>),
        With<TargetBlock>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_manager: Res<PlayerManager>,
    config: Res<GameConfiguration>,
    time: Res<Time>,
) {
    // Animate player blocks
    for (anim, material_handle, player_block) in animating_blocks.iter() {
        let elapsed = time.elapsed_secs() - anim.start_time;
        let progress = (elapsed / config.game.victory_display_duration).clamp(0.0, 1.0);
        let eased = progress * progress * (3.0 - 2.0 * progress);

        if let Some(material) = materials.get_mut(&material_handle.0) {
            let player_config = player_manager.get_player(player_block.player).unwrap();
            let base = player_config.emissive;
            let anim_config = &config.animation.victory;

            if anim.is_winner {
                // SUPERNOVA! Brightness from config
                let target = LinearRgba::rgb(
                    base.red * anim_config.winner_brightness_multiplier,
                    base.green * anim_config.winner_brightness_multiplier,
                    base.blue * anim_config.winner_brightness_multiplier,
                );
                material.emissive = LinearRgba::rgb(
                    base.red + (target.red - base.red) * eased,
                    base.green + (target.green - base.green) * eased,
                    base.blue + (target.blue - base.blue) * eased,
                );
            } else {
                // Dim from config
                let target = LinearRgba::rgb(
                    base.red * anim_config.loser_brightness_multiplier,
                    base.green * anim_config.loser_brightness_multiplier,
                    base.blue * anim_config.loser_brightness_multiplier,
                );
                material.emissive = LinearRgba::rgb(
                    base.red + (target.red - base.red) * eased,
                    base.green + (target.green - base.green) * eased,
                    base.blue + (target.blue - base.blue) * eased,
                );
            }
        }
    }

    // Animate target block (dims and fades like losers)
    if let Ok((anim, material_handle)) = animating_target.single() {
        let elapsed = time.elapsed_secs() - anim.start_time;
        let progress = (elapsed / config.game.victory_display_duration).clamp(0.0, 1.0);
        let eased = progress * progress * (3.0 - 2.0 * progress);

        if let Some(material) = materials.get_mut(&material_handle.0) {
            // Get target's base emissive from config
            let base = config.rendering.target_emissive.to_linear();
            let anim_config = &config.animation.victory;

            // Target dims like loser blocks
            let target = LinearRgba::rgb(
                base.red * anim_config.loser_brightness_multiplier,
                base.green * anim_config.loser_brightness_multiplier,
                base.blue * anim_config.loser_brightness_multiplier,
            );
            material.emissive = LinearRgba::rgb(
                base.red + (target.red - base.red) * eased,
                base.green + (target.green - base.green) * eased,
                base.blue + (target.blue - base.blue) * eased,
            );
        }
    }
}

/// Transition from victory animation to fadeout
pub fn transition_to_fadeout(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    config: Res<GameConfiguration>,
    animating: Query<(Entity, &MeshMaterial3d<StandardMaterial>), With<VictoryAnimation>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    if game_state.phase != GamePhase::VictoryAnimation {
        return;
    }

    let time_in_phase = time.elapsed_secs() - game_state.phase_start_time;
    if time_in_phase >= config.game.victory_display_duration {
        println!("Victory animation complete. Fading out blocks...");

        for (entity, material_handle) in animating.iter() {
            // Enable alpha blending for transparency when we start fading
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.alpha_mode = AlphaMode::Blend;
            }

            commands.entity(entity).remove::<VictoryAnimation>();
            commands.entity(entity).insert(FadingOut {
                start_time: time.elapsed_secs(),
                duration: config.game.fade_out_duration,
            });
        }

        game_state.transition_to_phase(GamePhase::FadingOut, time.elapsed_secs());
    }
}

/// Animate block fade-in (smooth appearance)
pub fn fade_in_blocks(
    mut commands: Commands,
    mut fading: Query<(
        Entity,
        &FadingIn,
        &MeshMaterial3d<StandardMaterial>,
        Option<&PlayerBlock>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_manager: Res<PlayerManager>,
    config: Res<GameConfiguration>,
    time: Res<Time>,
) {
    for (entity, fade, material_handle, player_block) in fading.iter_mut() {
        let elapsed = time.elapsed_secs() - fade.start_time;
        let progress = (elapsed / fade.duration).clamp(0.0, 1.0);

        if progress < 1.0 {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                // Enable alpha blending for transparency
                material.alpha_mode = AlphaMode::Blend;

                // Get the BASE emissive value from config/player (not from current material state)
                let base_emissive = if let Some(pb) = player_block {
                    // Player block - get emissive from player config
                    player_manager.get_player(pb.player).unwrap().emissive
                } else {
                    // Target block - get emissive from rendering config
                    config.rendering.target_emissive.to_linear()
                };

                // Smooth fade from start_alpha to 1.0
                let alpha = fade.start_alpha + (1.0 - fade.start_alpha) * progress;
                let emissive_strength = progress;

                // Fade in both color and emissive
                material.base_color.set_alpha(alpha);
                material.emissive = LinearRgba::rgb(
                    base_emissive.red * emissive_strength,
                    base_emissive.green * emissive_strength,
                    base_emissive.blue * emissive_strength,
                );
            }
        } else {
            // Fade complete - remove component and ensure full opacity
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.base_color.set_alpha(1.0);
                material.alpha_mode = AlphaMode::Opaque;
            }
            commands.entity(entity).remove::<FadingIn>();
        }
    }
}

/// Animate block fadeout
pub fn fade_out_blocks(
    mut commands: Commands,
    mut fading: Query<(Entity, &FadingOut, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, fade, material_handle) in fading.iter_mut() {
        let elapsed = time.elapsed_secs() - fade.start_time;
        let progress = (elapsed / fade.duration).clamp(0.0, 1.0);

        if progress < 1.0 {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                // Enable alpha blending for transparency
                material.alpha_mode = AlphaMode::Blend;

                let current_emissive = material.emissive;
                let alpha = 1.0 - progress;
                let emissive_strength = 1.0 - progress;

                // Fade both color and emissive
                material.base_color.set_alpha(alpha);
                material.emissive = LinearRgba::rgb(
                    current_emissive.red * emissive_strength,
                    current_emissive.green * emissive_strength,
                    current_emissive.blue * emissive_strength,
                );
            }
        } else {
            commands.entity(entity).despawn();
        }
    }
}

/// Start match victory animation when match ends
#[allow(clippy::type_complexity)]
pub fn start_match_victory_animation(
    mut commands: Commands,
    game_state: Res<GameState>,
    player_blocks: Query<(Entity, &PlayerBlock), (With<Block>, Without<MatchVictoryAnimation>)>,
    time: Res<Time>,
) {
    if game_state.phase != GamePhase::MatchComplete {
        return;
    }

    let time_in_phase = time.elapsed_secs() - game_state.phase_start_time;
    if time_in_phase > 0.1 {
        // Only run once
        return;
    }

    // Determine match winner
    let winner_player = if game_state.round_wins[0] > game_state.round_wins[1] {
        0
    } else {
        1
    };

    println!(
        "Starting match victory animation for Player {}",
        winner_player + 1
    );

    // Mark all blocks with match victory animation
    for (entity, _) in player_blocks.iter() {
        commands
            .entity(entity)
            .insert(MatchVictoryAnimation { winner_player });
    }
}

/// Animate match victory - ALL winner's blocks go supernova (200% brightness, 200% bloom)
pub fn animate_match_victory(
    animating_blocks: Query<(
        &MatchVictoryAnimation,
        &MeshMaterial3d<StandardMaterial>,
        &PlayerBlock,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_manager: Res<PlayerManager>,
    config: Res<GameConfiguration>,
) {
    for (anim, material_handle, player_block) in animating_blocks.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let player_config = player_manager.get_player(player_block.player).unwrap();
            let base_emissive = player_config.emissive;

            let is_winner = player_block.player == anim.winner_player;

            if is_winner {
                // Winner blocks: 400% brightness (4x multiplier)
                let multiplier = 4.0;
                material.emissive = LinearRgba::rgb(
                    base_emissive.red * multiplier,
                    base_emissive.green * multiplier,
                    base_emissive.blue * multiplier,
                );
            } else {
                // Loser blocks: 1% brightness (very dim)
                let multiplier = config.animation.victory.loser_brightness_multiplier;
                material.emissive = LinearRgba::rgb(
                    base_emissive.red * multiplier,
                    base_emissive.green * multiplier,
                    base_emissive.blue * multiplier,
                );
            }
        }
    }
}

/// Handle match completion and restart
pub fn handle_match_complete(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    config: Res<GameConfiguration>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    all_blocks: Query<Entity, With<Block>>,
) {
    if game_state.phase != GamePhase::MatchComplete {
        return;
    }

    let time_in_phase = time.elapsed_secs() - game_state.phase_start_time;

    if time_in_phase > config.game.match_complete_delay && keyboard.just_pressed(KeyCode::Space) {
        println!("\n=== STARTING NEW MATCH ===\n");

        // Despawn all blocks from previous match
        for entity in all_blocks.iter() {
            commands.entity(entity).despawn();
        }

        game_state.round_wins = [0, 0];
        game_state.starting_player = 1; // Will alternate to 0 in start_new_round
        game_state.start_new_round(&config);
    }
}
