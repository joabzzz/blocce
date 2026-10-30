// AI player logic for CPU opponents
use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::core::{
    ActiveBlock, Block, CameraTarget, GameConfiguration, GamePhase, GameState, OutOfBounds,
    PlayerBlock, PlayerManager, TargetBlock, ThrowingBlock,
};
use crate::gameplay::game::throw_block;
use crate::physics::calculate_aim_rotation;

// Type aliases for complex query types
type PlayerBlockQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static PlayerBlock), (With<Block>, Without<OutOfBounds>)>;

/// Performance data for a single throw
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are used for logging and future analysis
pub struct ThrowPerformanceData {
    // Intended throw parameters
    pub intended_rotation: f32,
    pub intended_power: f32,
    pub intended_target: Vec3,
    pub throw_time: f32,

    // Actual results
    pub landing_position: Vec3,
    pub final_distance_to_target: f32,
    pub actual_distance_traveled: f32,

    // Physics performance
    pub first_bounce_position: Option<Vec3>,
    pub total_bounces: u32,
    pub max_velocity_on_landing: f32,
    pub settling_time: f32, // Time to settle after throw

    // Accuracy metrics
    pub horizontal_error: f32, // Distance from intended target (XZ plane)
    pub distance_error: f32,   // Over/undershoot along throw direction
}

impl ThrowPerformanceData {
    /// Calculate power adjustment factor based on distance error
    /// Returns a multiplier to adjust future power (1.0 = no change)
    pub fn power_adjustment(&self) -> f32 {
        // If we overshot, reduce power. If undershot, increase power
        let error_ratio = self.distance_error / self.actual_distance_traveled.max(1.0);

        // Clamp adjustment to ±10% per throw to avoid wild swings
        let adjustment = (-error_ratio * 0.1).clamp(-0.1, 0.1);
        1.0 + adjustment
    }

    /// Calculate rotation adjustment based on horizontal error
    /// Returns degrees to adjust rotation
    pub fn rotation_adjustment(&self) -> f32 {
        // Calculate angle offset based on horizontal error
        if self.actual_distance_traveled < 1.0 {
            return 0.0; // Too close to make meaningful adjustment
        }

        let angle_error = (self.horizontal_error / self.actual_distance_traveled)
            .atan()
            .to_degrees();

        // Clamp to ±3 degrees per throw
        angle_error.clamp(-3.0, 3.0)
    }

    /// Get overall accuracy score (0.0 = terrible, 1.0 = perfect)
    pub fn accuracy_score(&self) -> f32 {
        let target_error = self.final_distance_to_target;
        // Score inversely proportional to error (perfect = 0 error)
        (1.0 / (1.0 + target_error)).powf(0.5)
    }
}

/// Component to mark a block for throw tracking
#[derive(Component)]
pub struct TrackedThrow {
    pub player: usize,
    pub intended_rotation: f32,
    pub intended_power: f32,
    pub intended_target: Vec3,
    pub throw_time: f32,
    pub first_bounce_position: Option<Vec3>,
    pub bounce_count: u32,
    pub max_landing_velocity: f32,
    pub last_position: Vec3,
    pub has_bounced: bool,
}

/// Resource storing throw history for each player
#[derive(Resource, Default)]
pub struct ThrowHistory {
    pub player_throws: [Vec<ThrowPerformanceData>; 2],
}

impl ThrowHistory {
    /// Add a completed throw to history
    pub fn add_throw(&mut self, player: usize, data: ThrowPerformanceData) {
        // Keep last 10 throws per player for memory efficiency
        if self.player_throws[player].len() >= 10 {
            self.player_throws[player].remove(0);
        }
        self.player_throws[player].push(data);
    }

    /// Get average power adjustment from recent throws
    pub fn average_power_adjustment(&self, player: usize, count: usize) -> f32 {
        let throws = &self.player_throws[player];
        if throws.is_empty() {
            return 1.0;
        }

        let recent = throws.iter().rev().take(count);
        let adjustments: Vec<f32> = recent.map(|t| t.power_adjustment()).collect();

        if adjustments.is_empty() {
            1.0
        } else {
            adjustments.iter().sum::<f32>() / adjustments.len() as f32
        }
    }

    /// Get average rotation adjustment from recent throws
    pub fn average_rotation_adjustment(&self, player: usize, count: usize) -> f32 {
        let throws = &self.player_throws[player];
        if throws.is_empty() {
            return 0.0;
        }

        let recent = throws.iter().rev().take(count);
        let adjustments: Vec<f32> = recent.map(|t| t.rotation_adjustment()).collect();

        if adjustments.is_empty() {
            0.0
        } else {
            adjustments.iter().sum::<f32>() / adjustments.len() as f32
        }
    }

    /// Get average accuracy from recent throws
    pub fn average_accuracy(&self, player: usize, count: usize) -> f32 {
        let throws = &self.player_throws[player];
        if throws.is_empty() {
            return 0.5; // Neutral score
        }

        let recent = throws.iter().rev().take(count);
        let scores: Vec<f32> = recent.map(|t| t.accuracy_score()).collect();

        if scores.is_empty() {
            0.5
        } else {
            scores.iter().sum::<f32>() / scores.len() as f32
        }
    }

    /// Clear history for a new round
    #[allow(dead_code)] // Reserved for future round reset functionality
    pub fn clear(&mut self) {
        self.player_throws[0].clear();
        self.player_throws[1].clear();
    }
}

/// AI thinking timer - adds realistic delay before CPU throws
#[derive(Resource)]
pub struct AIThinkingTimer {
    pub timer: f32,
    pub is_thinking: bool,
}

impl Default for AIThinkingTimer {
    fn default() -> Self {
        Self {
            timer: 0.0,
            is_thinking: false,
        }
    }
}

impl AIThinkingTimer {
    /// Start thinking with a random delay between min and max seconds
    pub fn start_thinking(&mut self, min_delay: f32, max_delay: f32) {
        let mut rng = rand::thread_rng();
        self.timer = rng.gen_range(min_delay..=max_delay);
        self.is_thinking = true;
    }

    /// Update timer and return true if thinking is complete
    pub fn update(&mut self, delta: f32) -> bool {
        if !self.is_thinking {
            return false;
        }

        self.timer -= delta;
        if self.timer <= 0.0 {
            self.is_thinking = false;
            return true;
        }
        false
    }

    pub fn reset(&mut self) {
        self.timer = 0.0;
        self.is_thinking = false;
    }
}

/// AI decision result containing throw parameters
#[derive(Debug, Clone)]
pub struct AIDecision {
    pub rotation: f32,
    pub power: f32,
    pub strategy: String, // For debugging/logging
}

/// Difficulty levels for AI
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum AIDifficulty {
    Easy,
    Medium,
    Hard,
    Expert, // Shooter
}

/// Block position data for analysis
#[derive(Debug, Clone)]
pub struct BlockInfo {
    position: Vec3,
    player: usize,
    distance_to_target: f32,
}

/// Analyze game state and make AI decision
/// This is the main AI brain - "Shooter" expert AI
pub fn make_ai_decision(
    game_state: &GameState,
    target_pos: Vec3,
    config: &GameConfiguration,
    all_blocks: &[BlockInfo],
    difficulty: AIDifficulty,
    throw_history: Option<&ThrowHistory>,
) -> AIDecision {
    match difficulty {
        AIDifficulty::Easy => make_easy_decision(config),
        AIDifficulty::Medium => make_medium_decision(target_pos, config),
        AIDifficulty::Hard => make_hard_decision(game_state, target_pos, config, all_blocks),
        AIDifficulty::Expert => {
            make_shooter_decision(game_state, target_pos, config, all_blocks, throw_history)
        }
    }
}

/// Easy AI - Random throws
fn make_easy_decision(config: &GameConfiguration) -> AIDecision {
    let mut rng = rand::thread_rng();

    let rotation = rng.gen_range(config.throwing.min_rotation..=config.throwing.max_rotation);
    let power = rng.gen_range(0.6..=0.9);

    AIDecision {
        rotation,
        power,
        strategy: "Random throw".to_string(),
    }
}

/// Medium AI - Aims at target with some error
fn make_medium_decision(target_pos: Vec3, config: &GameConfiguration) -> AIDecision {
    let mut rng = rand::thread_rng();

    let throw_pos = Vec3::new(
        config.arena.throwing_line.x,
        config.arena.throwing_line.y,
        config.arena.throwing_line.z,
    );

    // Calculate optimal rotation to face target
    let optimal_rotation = calculate_aim_rotation(throw_pos, target_pos);

    // Add ±10° error
    let rotation_error = rng.gen_range(-10.0..=10.0);
    let rotation = (optimal_rotation + rotation_error)
        .clamp(config.throwing.min_rotation, config.throwing.max_rotation);

    // Calculate distance and estimate power
    let distance = (target_pos - throw_pos).length();
    let optimal_power = estimate_power_for_distance(distance, config);

    // Add 15% power variance
    let power_variance = rng.gen_range(-0.15..=0.15);
    let power = (optimal_power + power_variance)
        .clamp(config.throwing.min_power, config.throwing.max_power);

    AIDecision {
        rotation,
        power,
        strategy: "Aiming at target (±10° error, ±15% power variance)".to_string(),
    }
}

/// Hard AI - Precise calculations with minimal error
fn make_hard_decision(
    game_state: &GameState,
    target_pos: Vec3,
    config: &GameConfiguration,
    all_blocks: &[BlockInfo],
) -> AIDecision {
    let mut rng = rand::thread_rng();

    let throw_pos = Vec3::new(
        config.arena.throwing_line.x,
        config.arena.throwing_line.y,
        config.arena.throwing_line.z,
    );

    let current_player = game_state.current_player;
    let opponent = 1 - current_player;

    // Determine if we're winning or losing
    let am_winning = game_state.scores[current_player] < game_state.scores[opponent]
        && game_state.scores[current_player] > 0.0;

    let aim_target = if am_winning && !all_blocks.is_empty() {
        // Defensive: Try to block opponent's best block or place conservatively
        let opponent_blocks: Vec<_> = all_blocks.iter().filter(|b| b.player == opponent).collect();

        if let Some(best_opponent_block) = opponent_blocks.iter().min_by(|a, b| {
            a.distance_to_target
                .partial_cmp(&b.distance_to_target)
                .unwrap()
        }) {
            // Aim slightly between opponent's block and target
            target_pos.lerp(best_opponent_block.position, 0.3)
        } else {
            target_pos
        }
    } else {
        // Offensive: Aim directly at target
        target_pos
    };

    let optimal_rotation = calculate_aim_rotation(throw_pos, aim_target);

    // Add minimal ±3° physics variance
    let rotation_error = rng.gen_range(-3.0..=3.0);
    let rotation = (optimal_rotation + rotation_error)
        .clamp(config.throwing.min_rotation, config.throwing.max_rotation);

    let distance = (aim_target - throw_pos).length();
    let power = calculate_optimal_power(distance, config)
        .clamp(config.throwing.min_power, config.throwing.max_power);

    let strategy = if am_winning {
        "Defensive: Blocking opponent's best position".to_string()
    } else {
        "Offensive: Aiming for target center".to_string()
    };

    AIDecision {
        rotation,
        power,
        strategy,
    }
}

/// Expert AI "Shooter" - Advanced strategic thinking with performance learning
fn make_shooter_decision(
    game_state: &GameState,
    target_pos: Vec3,
    config: &GameConfiguration,
    all_blocks: &[BlockInfo],
    throw_history: Option<&ThrowHistory>,
) -> AIDecision {
    let mut rng = rand::thread_rng();

    let throw_pos = Vec3::new(
        config.arena.throwing_line.x,
        config.arena.throwing_line.y,
        config.arena.throwing_line.z,
    );

    let current_player = game_state.current_player;
    let opponent = 1 - current_player;

    // Advanced game state analysis
    let blocks_remaining = config.game.blocks_per_player - game_state.blocks_thrown[current_player];
    let opponent_blocks_remaining =
        config.game.blocks_per_player - game_state.blocks_thrown[opponent];

    let my_best_distance = game_state.scores[current_player];
    let opponent_best_distance = game_state.scores[opponent];

    let round_lead =
        game_state.round_wins[current_player] as i32 - game_state.round_wins[opponent] as i32;

    // Separate blocks by player
    let my_blocks: Vec<_> = all_blocks
        .iter()
        .filter(|b| b.player == current_player)
        .collect();
    let opponent_blocks: Vec<_> = all_blocks.iter().filter(|b| b.player == opponent).collect();

    // Find opponent's best block
    let opponent_best_block = opponent_blocks.iter().min_by(|a, b| {
        a.distance_to_target
            .partial_cmp(&b.distance_to_target)
            .unwrap()
    });

    // Strategic decision tree
    let (aim_target, strategy) = if round_lead >= 2 && my_best_distance < opponent_best_distance {
        // Large lead and winning: Play VERY conservatively
        // Aim for a safe zone slightly off-target
        let safe_offset = Vec3::new(rng.gen_range(-2.0..=2.0), 0.0, rng.gen_range(-2.0..=2.0));
        (
            target_pos + safe_offset,
            "Conservative: Large lead, playing safe".to_string(),
        )
    } else if blocks_remaining == 1 && my_best_distance > opponent_best_distance {
        // Last block and losing: High-risk knock-away shot
        if let Some(best_opp_block) = opponent_best_block {
            // Try to knock away opponent's best block
            // Aim at the block itself with high power
            (
                best_opp_block.position,
                "Desperation: Knock-away shot at opponent's best block".to_string(),
            )
        } else {
            (target_pos, "Last block: Going for target".to_string())
        }
    } else if (my_best_distance > 0.0 || !my_blocks.is_empty())
        && my_best_distance < opponent_best_distance
        && opponent_blocks_remaining > 0
    {
        // Winning but opponent has blocks left: Create blocking wall
        if let Some(best_opp_block) = opponent_best_block {
            // Place block between opponent's best and target
            let block_position = target_pos.lerp(best_opp_block.position, 0.4);
            (
                block_position,
                "Defensive: Creating blocking wall".to_string(),
            )
        } else {
            (target_pos, "Defensive: Maintaining position".to_string())
        }
    } else if my_blocks.len() >= 2
        && (my_best_distance < opponent_best_distance || opponent_best_distance == 0.0)
    {
        // Have multiple blocks and winning: Cluster strategy
        // Place blocks near our best block to create a cluster
        let our_best = my_blocks
            .iter()
            .min_by(|a, b| {
                a.distance_to_target
                    .partial_cmp(&b.distance_to_target)
                    .unwrap()
            })
            .unwrap();

        let cluster_offset = Vec3::new(rng.gen_range(-1.5..=1.5), 0.0, rng.gen_range(-1.5..=1.5));
        (
            our_best.position + cluster_offset,
            "Defensive: Clustering around our best block".to_string(),
        )
    } else {
        // Default: Offensive - aim for target with slight variance
        let precision_offset = Vec3::new(rng.gen_range(-0.5..=0.5), 0.0, rng.gen_range(-0.5..=0.5));
        (
            target_pos + precision_offset,
            "Offensive: Precision shot at target".to_string(),
        )
    };

    // Calculate rotation
    let optimal_rotation = calculate_aim_rotation(throw_pos, aim_target);

    // Shooter has excellent accuracy: ±2° variance only
    // Add slight randomness to prevent deterministic outcomes
    let rotation_error = rng.gen_range(-2.0..=2.0);
    let mut rotation = (optimal_rotation + rotation_error)
        .clamp(config.throwing.min_rotation, config.throwing.max_rotation);

    // Add small power variance (±2%) to prevent identical throws
    let power_variance = rng.gen_range(-0.02..=0.02);

    // Calculate power
    let distance = (aim_target - throw_pos).length();
    let mut power = calculate_optimal_power(distance, config);

    // Apply base power variance before strategy adjustments
    power = (power * (1.0 + power_variance)).max(config.throwing.min_power);

    // On knock-away shots, use extra power
    if strategy.contains("Knock-away") {
        power = (power * 1.15).min(config.throwing.max_power);
    }

    // === LEARNING FROM THROW HISTORY ===
    if let Some(history) = throw_history {
        let current_player = game_state.current_player;

        // Apply learning adjustments based on last 3 throws
        let power_adjustment = history.average_power_adjustment(current_player, 3);
        let rotation_adjustment = history.average_rotation_adjustment(current_player, 3);
        let accuracy = history.average_accuracy(current_player, 5);

        // Only apply adjustments if we have enough data
        if !history.player_throws[current_player].is_empty() {
            power *= power_adjustment;
            rotation += rotation_adjustment;

            // Log learning adjustments
            if power_adjustment != 1.0 || rotation_adjustment != 0.0 {
                println!("  🧠 Learning from history:");
                println!(
                    "     Power adjustment: {:.1}%",
                    (power_adjustment - 1.0) * 100.0
                );
                println!("     Rotation adjustment: {:.1}°", rotation_adjustment);
                println!("     Recent accuracy: {:.1}%", accuracy * 100.0);
            }
        }
    }

    power = power.clamp(config.throwing.min_power, config.throwing.max_power);
    rotation = rotation.clamp(config.throwing.min_rotation, config.throwing.max_rotation);

    AIDecision {
        rotation,
        power,
        strategy,
    }
}

/// Estimate power needed for a given distance (simple heuristic)
fn estimate_power_for_distance(distance: f32, _config: &GameConfiguration) -> f32 {
    // Simple linear mapping: 10m = 0.5 power, 30m = 1.0 power
    let normalized = (distance - 10.0) / 20.0;
    normalized.clamp(0.5, 1.0)
}

/// Calculate optimal power using physics equations
fn calculate_optimal_power(distance: f32, config: &GameConfiguration) -> f32 {
    let fixed_angle = config.throwing.default_angle; // 45 degrees
    let gravity = config.physics.gravity.abs();
    let power_multiplier = config.throwing.power_multiplier;

    // More precise calculation using trajectory equation
    // For angle θ and distance d: v² = (g * d) / (sin(2θ) + 2*h/d*cos²(θ))
    let angle_rad = fixed_angle.to_radians();
    let sin_2theta = (2.0 * angle_rad).sin();

    // Simplified for approximately level ground
    let velocity_needed = ((gravity * distance) / sin_2theta).sqrt();

    // Convert velocity to power (0.0 to 1.0)
    let power = velocity_needed / power_multiplier;

    // Add slight overhead for air resistance compensation
    power * 1.05
}

/// System to collect block information for AI analysis
pub fn collect_block_info(
    player_blocks: &PlayerBlockQuery,
    target_transform: &Transform,
) -> Vec<BlockInfo> {
    let target_pos = target_transform.translation;

    player_blocks
        .iter()
        .map(|(transform, player_block)| {
            let pos = transform.translation;
            let distance = pos.distance(target_pos);

            BlockInfo {
                position: pos,
                player: player_block.player,
                distance_to_target: distance,
            }
        })
        .collect()
}

/// Main AI system - decides when CPU should throw
#[allow(clippy::too_many_arguments)]
pub fn ai_player_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_state: ResMut<GameState>,
    mut ai_timer: ResMut<AIThinkingTimer>,
    game_setup: Res<crate::core::GameSetup>,
    config: Res<GameConfiguration>,
    player_manager: Res<PlayerManager>,
    target_query: Query<&Transform, With<TargetBlock>>,
    player_blocks: PlayerBlockQuery,
    throwing_blocks: Query<Entity, With<ThrowingBlock>>,
    camera_targets: Query<Entity, With<CameraTarget>>,
    throw_history: Res<ThrowHistory>,
    time: Res<Time>,
) {
    // Only run during Playing phase when blocks have settled
    if game_state.phase != GamePhase::Playing || !game_state.all_blocks_settled {
        return;
    }

    let current_player = game_state.current_player;
    let blocks_per_player = config.game.blocks_per_player;

    // Check if current player has blocks remaining
    if game_state.blocks_thrown[current_player] >= blocks_per_player {
        return;
    }

    // Check if current player is CPU
    let is_cpu = match current_player {
        0 => game_setup.player1_type == crate::core::PlayerType::CPU,
        1 => game_setup.player2_type == crate::core::PlayerType::CPU,
        _ => false,
    };

    if !is_cpu {
        return;
    }

    // If AI is not thinking yet, start thinking
    if !ai_timer.is_thinking {
        ai_timer.start_thinking(1.0, 2.0); // 1-2 second delay
        println!("🤖 CPU Player {} is thinking...", current_player + 1);
        return;
    }

    // Update thinking timer
    if !ai_timer.update(time.delta_secs()) {
        return; // Still thinking...
    }

    // Thinking complete - make decision and throw!
    if let Ok(target_transform) = target_query.single() {
        let target_pos = target_transform.translation;
        let all_blocks = collect_block_info(&player_blocks, target_transform);

        // Use Expert difficulty (Shooter) for now
        let decision = make_ai_decision(
            &game_state,
            target_pos,
            &config,
            &all_blocks,
            AIDifficulty::Expert,
            Some(&throw_history),
        );

        println!(
            "🎯 CPU Player {} decided: {} | Rotation: {:.1}°, Power: {:.2}",
            current_player + 1,
            decision.strategy,
            decision.rotation,
            decision.power
        );

        // Set game state for the throw
        game_state.throw_rotation = decision.rotation;
        game_state.throw_power = decision.power;

        // Execute the throw
        throw_block(
            &mut commands,
            &mut meshes,
            &mut materials,
            &game_state,
            &player_manager,
            &config,
            &throwing_blocks,
            &camera_targets,
            &time,
        );

        // Attach tracking component to the thrown block
        // We need to find the newly spawned block - it will be the one with ActiveBlock and CameraTarget
        // This will be handled in a separate system right after throw

        // Update game state after throw
        let current_player = game_state.current_player;
        game_state.blocks_thrown[current_player] += 1;
        game_state.throw_power = config.throwing.min_power;
        game_state.all_blocks_settled = false;

        println!(
            "🤖 CPU Player {} threw block {}/{}",
            current_player + 1,
            game_state.blocks_thrown[current_player],
            config.game.blocks_per_player
        );
    }

    ai_timer.reset();
}

// =============================================================================
// THROW PERFORMANCE TRACKING
// =============================================================================

/// Attach tracking component to newly thrown blocks
#[allow(clippy::type_complexity)]
pub fn attach_throw_tracking(
    mut commands: Commands,
    game_state: Res<GameState>,
    target_query: Query<&Transform, With<TargetBlock>>,
    new_blocks: Query<
        (Entity, &PlayerBlock, &Transform),
        (With<ActiveBlock>, Without<TrackedThrow>),
    >,
    _config: Res<GameConfiguration>,
    time: Res<Time>,
) {
    let Ok(target_transform) = target_query.single() else {
        return;
    };

    let target_pos = target_transform.translation;

    for (entity, player_block, transform) in new_blocks.iter() {
        // Use the AI's target position (for CPU) or actual target (for human)
        // For now, we'll use the target position as the intended target
        let intended_target = target_pos;

        commands.entity(entity).insert(TrackedThrow {
            player: player_block.player,
            intended_rotation: game_state.throw_rotation,
            intended_power: game_state.throw_power,
            intended_target,
            throw_time: time.elapsed_secs(),
            first_bounce_position: None,
            bounce_count: 0,
            max_landing_velocity: 0.0,
            last_position: transform.translation,
            has_bounced: false,
        });
    }
}

/// Track bounce events for thrown blocks
#[allow(clippy::type_complexity)]
pub fn track_block_bounces(
    mut tracked_blocks: Query<(&Transform, &LinearVelocity, &mut TrackedThrow), With<ActiveBlock>>,
    _config: Res<GameConfiguration>,
) {
    for (transform, velocity, mut tracked) in tracked_blocks.iter_mut() {
        let current_pos = transform.translation;
        let speed = velocity.length();

        // Check if block just bounced (sudden position change + high velocity)
        let vertical_velocity = velocity.0.y.abs();

        // Detect bounce: block is moving up with significant velocity after being near ground
        if vertical_velocity > 2.0
            && current_pos.y < (tracked.last_position.y + 0.5)
            && !tracked.has_bounced
        {
            tracked.has_bounced = true;
            tracked.bounce_count += 1;

            // Record first bounce position (only logs once per throw)
            if tracked.first_bounce_position.is_none() {
                tracked.first_bounce_position = Some(current_pos);
            }

            // Track maximum landing velocity
            if speed > tracked.max_landing_velocity {
                tracked.max_landing_velocity = speed;
            }
        } else if vertical_velocity < 1.0 {
            // Reset bounce detection when moving slowly
            tracked.has_bounced = false;
        }

        // Update last position
        tracked.last_position = current_pos;
    }
}

/// Finalize throw performance data when block settles
#[allow(clippy::type_complexity)]
pub fn finalize_throw_performance(
    mut commands: Commands,
    mut throw_history: ResMut<ThrowHistory>,
    tracked_blocks: Query<
        (
            Entity,
            &Transform,
            &TrackedThrow,
            &LinearVelocity,
            &AngularVelocity,
        ),
        With<ActiveBlock>,
    >,
    target_query: Query<&Transform, With<TargetBlock>>,
    config: Res<GameConfiguration>,
    time: Res<Time>,
) {
    let settlement = &config.physics.settlement;
    let Ok(target_transform) = target_query.single() else {
        return;
    };
    let target_pos = target_transform.translation;

    for (entity, transform, tracked, lin_vel, ang_vel) in tracked_blocks.iter() {
        let speed = lin_vel.length();
        let ang_speed = ang_vel.length();

        // Check if block has settled
        if speed < settlement.velocity_threshold
            && ang_speed < settlement.angular_velocity_threshold
        {
            let landing_pos = transform.translation;
            let throw_pos = Vec3::new(
                config.arena.throwing_line.x,
                config.arena.throwing_line.y,
                config.arena.throwing_line.z,
            );

            // Calculate performance metrics
            let actual_distance_traveled = landing_pos.distance(throw_pos);
            let final_distance_to_target = landing_pos.distance(target_pos);

            // Calculate intended throw direction (XZ plane)
            let intended_dir_xz = Vec2::new(
                tracked.intended_target.x - throw_pos.x,
                tracked.intended_target.z - throw_pos.z,
            )
            .normalize();

            let actual_dir_xz =
                Vec2::new(landing_pos.x - throw_pos.x, landing_pos.z - throw_pos.z).normalize();

            // Horizontal error (perpendicular to throw direction)
            let cross = intended_dir_xz.x * actual_dir_xz.y - intended_dir_xz.y * actual_dir_xz.x;
            let horizontal_error = cross.abs() * actual_distance_traveled;

            // Distance error (along throw direction - positive = overshoot)
            let intended_distance = tracked.intended_target.distance(throw_pos);
            let distance_error = actual_distance_traveled - intended_distance;

            let settling_time = time.elapsed_secs() - tracked.throw_time;

            let performance = ThrowPerformanceData {
                intended_rotation: tracked.intended_rotation,
                intended_power: tracked.intended_power,
                intended_target: tracked.intended_target,
                throw_time: tracked.throw_time,
                landing_position: landing_pos,
                final_distance_to_target,
                actual_distance_traveled,
                first_bounce_position: tracked.first_bounce_position,
                total_bounces: tracked.bounce_count,
                max_velocity_on_landing: tracked.max_landing_velocity,
                settling_time,
                horizontal_error,
                distance_error,
            };

            // Performance tracking is done silently
            // Uncomment below for debugging throw performance:
            // println!("  📊 Throw Performance Summary:");
            // println!(
            //     "     Distance traveled: {:.2}m (intended: {:.2}m, error: {:.2}m)",
            //     actual_distance_traveled, intended_distance, distance_error
            // );
            // println!("     Horizontal error: {:.2}m", horizontal_error);
            // println!("     Bounces: {}", tracked.bounce_count);
            // println!(
            //     "     Final distance to target: {:.2}m",
            //     final_distance_to_target
            // );
            // println!(
            //     "     Accuracy score: {:.2}%",
            //     performance.accuracy_score() * 100.0
            // );
            // println!(
            //     "     Suggested power adjustment: {:.2}%",
            //     (performance.power_adjustment() - 1.0) * 100.0
            // );
            // println!(
            //     "     Suggested rotation adjustment: {:.1}°",
            //     performance.rotation_adjustment()
            // );

            throw_history.add_throw(tracked.player, performance);

            // Remove tracking component
            commands.entity(entity).remove::<TrackedThrow>();
        }
    }
}
