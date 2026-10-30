// Physics checks, distance tracking, and out-of-bounds handling
use avian3d::prelude::*;
use bevy::prelude::*;

use crate::core::{
    ActiveBlock, Block, CameraTarget, GameConfiguration, GameState, OutOfBounds, PlayerBlock,
    SettledBlock, TargetBlock,
};

/// Continuously track all blocks' distances to the target
#[allow(clippy::type_complexity)]
pub fn update_distances(
    mut game_state: ResMut<GameState>,
    player_blocks: Query<(&Transform, &PlayerBlock), (With<Block>, Without<OutOfBounds>)>,
    target_query: Query<&Transform, With<TargetBlock>>,
) {
    if let Ok(target_transform) = target_query.single() {
        let target_pos = target_transform.translation;

        // Track closest distance for each player
        let mut closest_distances = [f32::MAX, f32::MAX];

        for (transform, player_block) in player_blocks.iter() {
            let distance = transform.translation.distance(target_pos);
            let player_id = player_block.player;

            if distance < closest_distances[player_id] {
                closest_distances[player_id] = distance;
            }
        }

        // Update scores
        for (player_id, &closest_distance) in closest_distances.iter().enumerate() {
            if closest_distance < f32::MAX {
                game_state.scores[player_id] = closest_distance;
            } else if game_state.blocks_thrown[player_id] > 0 {
                game_state.scores[player_id] = f32::MAX; // All blocks out of bounds
            } else {
                game_state.scores[player_id] = 0.0; // No blocks thrown yet
            }
        }
    }
}

/// Check if blocks have fallen off the map
#[allow(clippy::type_complexity)]
pub fn check_out_of_bounds(
    mut commands: Commands,
    blocks: Query<(Entity, &Transform, &PlayerBlock), (With<Block>, Without<OutOfBounds>)>,
    config: Res<GameConfiguration>,
) {
    let boundaries = &config.arena.boundaries;

    for (entity, transform, _player_block) in blocks.iter() {
        let pos = transform.translation;

        // Boundaries from config
        let off_side = pos.x.abs() > boundaries.x_max || pos.z.abs() > boundaries.z_max;
        let fell_below = pos.y < boundaries.y_despawn_immediate;

        if fell_below {
            commands.entity(entity).insert(OutOfBounds);
            commands.entity(entity).remove::<ActiveBlock>();
            commands.entity(entity).despawn();
        } else if off_side {
            commands.entity(entity).insert(OutOfBounds);
            commands.entity(entity).remove::<ActiveBlock>();
        }
    }
}

/// Despawn blocks that are out of bounds and stopped moving
#[allow(clippy::type_complexity)]
pub fn cleanup_out_of_bounds(
    mut commands: Commands,
    blocks: Query<(Entity, &Transform, &LinearVelocity), (With<Block>, With<OutOfBounds>)>,
    config: Res<GameConfiguration>,
) {
    let boundaries = &config.arena.boundaries;

    for (entity, transform, velocity) in blocks.iter() {
        let speed = velocity.length();
        if speed < boundaries.oob_despawn_velocity
            || transform.translation.y < boundaries.y_despawn_cleanup
        {
            commands.entity(entity).despawn();
        }
    }
}

/// Detect when camera target blocks have settled and mark them
#[allow(clippy::type_complexity)]
pub fn detect_settled_camera_target(
    mut commands: Commands,
    camera_targets: Query<
        (Entity, &LinearVelocity, &AngularVelocity),
        (With<CameraTarget>, Without<SettledBlock>),
    >,
    config: Res<GameConfiguration>,
    time: Res<Time>,
) {
    let velocity_threshold = config.physics.settlement.velocity_threshold;

    for (entity, lin_vel, ang_vel) in camera_targets.iter() {
        let speed = lin_vel.length();
        let ang_speed = ang_vel.length();

        // If block is moving slowly enough, mark it as settled
        if speed < velocity_threshold && ang_speed < velocity_threshold {
            commands.entity(entity).insert(SettledBlock {
                settled_time: time.elapsed_secs(),
            });
        }
    }
}

/// Remove camera target after block has been settled for a delay period
pub fn remove_camera_target_after_settle(
    mut commands: Commands,
    settled_targets: Query<(Entity, &SettledBlock), With<CameraTarget>>,
    time: Res<Time>,
) {
    let settle_view_duration = 2.0; // Seconds to view settled block before returning camera

    for (entity, settled) in settled_targets.iter() {
        let time_since_settled = time.elapsed_secs() - settled.settled_time;

        if time_since_settled >= settle_view_duration {
            // Remove both CameraTarget and SettledBlock components
            commands.entity(entity).remove::<CameraTarget>();
            commands.entity(entity).remove::<SettledBlock>();
        }
    }
}
