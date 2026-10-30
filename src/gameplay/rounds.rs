// Round progression, scoring, and round management
use avian3d::prelude::*;
use bevy::prelude::*;

use super::arena::spawn_target;
use crate::core::{
    ActiveBlock, Block, FadingOut, GameConfiguration, GamePhase, GameState, OutOfBounds,
    PlayerBlock, TargetBlock,
};

/// Check if blocks have settled after a throw (during Playing phase)
pub fn check_throw_settled(
    mut game_state: ResMut<GameState>,
    config: Res<GameConfiguration>,
    blocks: Query<(&LinearVelocity, &AngularVelocity), With<Block>>,
    out_of_bounds: Query<Entity, (With<Block>, With<OutOfBounds>)>,
    time: Res<Time>,
) {
    // Only check during Playing phase when blocks are not settled
    if game_state.phase != GamePhase::Playing || game_state.all_blocks_settled {
        return;
    }

    // Check if any blocks exist and if they're all settled
    if blocks.is_empty() {
        game_state.all_blocks_settled = true;
        game_state.settling_start_time = None;
        return;
    }

    let settlement = &config.physics.settlement;

    let mut all_settled = true;
    for (lin_vel, ang_vel) in blocks.iter() {
        let speed = lin_vel.length();
        let ang_speed = ang_vel.length();
        if speed > settlement.velocity_threshold
            || ang_speed > settlement.angular_velocity_threshold
        {
            all_settled = false;
            break;
        }
    }

    // Wait for OOB blocks to despawn
    if out_of_bounds.iter().count() > 0 {
        all_settled = false;
    }

    if !all_settled {
        game_state.settling_start_time = None;
        return;
    }

    // Track settling time
    let current_time = time.elapsed_secs();
    if game_state.settling_start_time.is_none() {
        game_state.settling_start_time = Some(current_time);
        return;
    }

    let time_settled = current_time - game_state.settling_start_time.unwrap();
    if time_settled < config.game.settle_duration {
        return;
    }

    // Blocks settled after this throw!
    game_state.all_blocks_settled = true;
    game_state.settling_start_time = None;

    println!("Blocks settled. Ready for next throw.");

    let blocks_per_player = config.game.blocks_per_player;

    // Check if current player finished their turns
    if game_state.blocks_thrown[game_state.current_player] >= blocks_per_player {
        // Check if other player also finished
        let other_player = 1 - game_state.current_player;
        if game_state.blocks_thrown[other_player] >= blocks_per_player {
            // Both players done - move to BlocksSettling phase for final scoring
            println!("\n=== ALL BLOCKS THROWN ===");
            println!("Waiting for final settlement...");
            game_state.transition_to_phase(GamePhase::BlocksSettling, time.elapsed_secs());
        } else {
            // Switch to other player
            game_state.current_player = other_player;
            game_state.throw_rotation = config.throwing.default_rotation;
            game_state.throw_power = config.throwing.min_power;
            println!(
                "\n=== SWITCHING TO PLAYER {} ===",
                game_state.current_player + 1
            );
        }
    }
}

/// Check if round is complete (both players threw all blocks each)
pub fn check_round_complete(
    mut game_state: ResMut<GameState>,
    config: Res<GameConfiguration>,
    time: Res<Time>,
) {
    if game_state.phase != GamePhase::Playing {
        return;
    }

    let blocks_per_player = config.game.blocks_per_player;

    // This is now handled by check_throw_settled, but keep as fallback
    if game_state.blocks_thrown[0] >= blocks_per_player
        && game_state.blocks_thrown[1] >= blocks_per_player
        && game_state.all_blocks_settled
    {
        println!("\n=== ALL BLOCKS THROWN ===");
        println!("Waiting for blocks to settle...");
        game_state.transition_to_phase(GamePhase::BlocksSettling, time.elapsed_secs());
    }
}

/// Check if all blocks have settled
#[allow(clippy::too_many_arguments)]
pub fn check_blocks_settled(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    config: Res<GameConfiguration>,
    blocks: Query<(&LinearVelocity, &AngularVelocity), With<Block>>,
    target_query: Query<&Transform, With<TargetBlock>>,
    player_blocks: Query<(&Transform, &PlayerBlock), Without<OutOfBounds>>,
    out_of_bounds: Query<Entity, (With<Block>, With<OutOfBounds>)>,
    active_blocks: Query<Entity, With<ActiveBlock>>,
    time: Res<Time>,
) {
    if game_state.phase != GamePhase::BlocksSettling {
        return;
    }

    let settlement = &config.physics.settlement;

    // Check velocities
    let mut all_settled = true;
    for (lin_vel, ang_vel) in blocks.iter() {
        let speed = lin_vel.length();
        let ang_speed = ang_vel.length();
        if speed > settlement.velocity_threshold
            || ang_speed > settlement.angular_velocity_threshold
        {
            all_settled = false;
            break;
        }
    }

    // Wait for OOB blocks to despawn
    if out_of_bounds.iter().count() > 0 {
        all_settled = false;
    }

    if !all_settled || blocks.iter().count() == 0 {
        game_state.settling_start_time = None;
        return;
    }

    // Track settling time
    let current_time = time.elapsed_secs();
    if game_state.settling_start_time.is_none() {
        game_state.settling_start_time = Some(current_time);
        return;
    }

    let time_settled = current_time - game_state.settling_start_time.unwrap();
    if time_settled < config.game.settle_duration {
        return;
    }

    // Blocks settled! Calculate final scores
    println!("\n=== BLOCKS SETTLED ===");
    game_state.settling_start_time = None;

    // Remove ActiveBlock component
    for entity in active_blocks.iter() {
        commands.entity(entity).remove::<ActiveBlock>();
    }

    // Calculate final scores
    if let Ok(target_transform) = target_query.single() {
        let target_pos = target_transform.translation;

        for player_id in 0..2 {
            let mut closest_distance = f32::MAX;
            let mut blocks_in_play = 0;

            for (transform, player_block) in player_blocks.iter() {
                if player_block.player == player_id {
                    blocks_in_play += 1;
                    let distance = transform.translation.distance(target_pos);
                    if distance < closest_distance {
                        closest_distance = distance;
                    }
                }
            }

            if closest_distance < f32::MAX {
                game_state.scores[player_id] = closest_distance;
                println!(
                    "Player {}: {:.2} units ({} blocks in play)",
                    player_id + 1,
                    closest_distance,
                    blocks_in_play
                );
            } else {
                game_state.scores[player_id] = f32::MAX;
                println!("Player {} - ALL BLOCKS FELL OFF!", player_id + 1);
            }
        }
    }

    // Determine winner
    determine_round_winner(&mut game_state, &config, &time);
}

/// Determine round winner and update match score
#[allow(clippy::if_same_then_else)]
fn determine_round_winner(
    game_state: &mut GameState,
    config: &GameConfiguration,
    time: &Res<Time>,
) {
    let round_winner =
        if game_state.scores[0] >= f32::MAX - 1.0 && game_state.scores[1] >= f32::MAX - 1.0 {
            None // Both lost all blocks
        } else if game_state.scores[0] >= f32::MAX - 1.0 {
            Some(1) // P1 lost all blocks
        } else if game_state.scores[1] >= f32::MAX - 1.0 {
            Some(0) // P2 lost all blocks
        } else if game_state.scores[0] < game_state.scores[1] {
            Some(0) // P1 closer
        } else if game_state.scores[0] > game_state.scores[1] {
            Some(1) // P2 closer
        } else {
            None // Tie
        };

    println!("\n=== ROUND COMPLETE ===");

    if let Some(winner) = round_winner {
        game_state.round_wins[winner] += 1;
        game_state.round_winner = Some(winner);
        println!("🏆 Player {} wins this round!", winner + 1);
    } else {
        game_state.round_winner = None;
        println!("Round tied!");
    }

    println!("\n📊 MATCH SCORE (Best of 7)");
    println!(
        "Player 1: {} | Player 2: {}",
        game_state.round_wins[0], game_state.round_wins[1]
    );

    // Check if someone won the match
    if game_state.round_wins[0] >= config.game.rounds_to_win {
        println!("\n🎉 MATCH OVER! Player 1 WINS! 🎉\n");
        game_state.transition_to_phase(GamePhase::MatchComplete, time.elapsed_secs());
    } else if game_state.round_wins[1] >= config.game.rounds_to_win {
        println!("\n🎉 MATCH OVER! Player 2 WINS! 🎉\n");
        game_state.transition_to_phase(GamePhase::MatchComplete, time.elapsed_secs());
    } else {
        game_state.transition_to_phase(GamePhase::VictoryAnimation, time.elapsed_secs());
    }
}

/// Start next round after all blocks faded
pub fn start_next_round(
    mut game_state: ResMut<GameState>,
    config: Res<GameConfiguration>,
    fading: Query<Entity, With<FadingOut>>,
) {
    if game_state.phase != GamePhase::FadingOut {
        return;
    }

    if fading.is_empty() {
        game_state.start_new_round(&config);
    }
}

/// Respawn target block at new random location for each round
pub fn respawn_target_for_round(
    mut commands: Commands,
    game_state: Res<GameState>,
    config: Res<GameConfiguration>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    target_query: Query<Entity, With<TargetBlock>>,
    time: Res<Time>,
) {
    // Only respawn when entering Playing phase and no target exists
    if game_state.phase == GamePhase::Playing && target_query.is_empty() {
        println!("Spawning new target at random location...");
        spawn_target(&mut commands, &mut meshes, &mut materials, &config, &time);
    }
}
