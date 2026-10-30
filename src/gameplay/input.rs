// Input handling and block throwing
use bevy::prelude::*;

use crate::core::{
    CameraTarget, GameConfiguration, GamePhase, GameSetup, GameState, PlayerBlockBundle,
    PlayerManager, PlayerType, ThrowingBlock,
};
use crate::physics::ThrowParameters;

/// Handle player input for aiming and throwing
#[allow(clippy::too_many_arguments)]
pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    setup: Res<GameSetup>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_manager: Res<PlayerManager>,
    query: Query<Entity, With<ThrowingBlock>>,
    camera_targets: Query<Entity, With<CameraTarget>>,
    config: Res<GameConfiguration>,
    time: Res<Time>,
) {
    // Only allow input during Playing phase
    let player_type = if game_state.current_player == 0 {
        setup.player1_type
    } else {
        setup.player2_type
    };
    if game_state.phase != GamePhase::Playing
        || !game_state.all_blocks_settled
        || player_type == PlayerType::CPU
    {
        return;
    }

    let throwing_cfg = &config.throwing;
    let blocks_per_player = config.game.blocks_per_player;

    // Check if current player has blocks remaining
    if game_state.blocks_thrown[game_state.current_player] >= blocks_per_player {
        return;
    }

    // Adjust aim rotation with A/D keys (A = left, D = right)
    if keyboard.pressed(KeyCode::KeyA) {
        game_state.throw_rotation -= throwing_cfg.rotation_speed * time.delta_secs();
        game_state.throw_rotation = game_state.throw_rotation.max(throwing_cfg.min_rotation);
    }
    if keyboard.pressed(KeyCode::KeyD) {
        game_state.throw_rotation += throwing_cfg.rotation_speed * time.delta_secs();
        game_state.throw_rotation = game_state.throw_rotation.min(throwing_cfg.max_rotation);
    }

    // Power control - hold space to charge
    if keyboard.pressed(KeyCode::Space) {
        game_state.throw_power += throwing_cfg.charge_rate * time.delta_secs();
        game_state.throw_power = game_state.throw_power.min(throwing_cfg.max_power);
    }

    // Release space to throw
    if keyboard.just_released(KeyCode::Space) {
        throw_block(
            &mut commands,
            &mut meshes,
            &mut materials,
            &game_state,
            &player_manager,
            &config,
            &query,
            &camera_targets,
            &time,
        );

        let current_player = game_state.current_player;
        game_state.blocks_thrown[current_player] += 1;
        game_state.throw_power = throwing_cfg.min_power;
        game_state.all_blocks_settled = false; // Mark that we need to wait for settling

        println!(
            "Player {} threw block {}/{}",
            current_player + 1,
            game_state.blocks_thrown[current_player],
            blocks_per_player
        );
    }
}

/// Spawn and throw a block with physics
#[allow(clippy::too_many_arguments)]
pub fn throw_block(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_state: &GameState,
    player_manager: &PlayerManager,
    config: &GameConfiguration,
    query: &Query<Entity, With<ThrowingBlock>>,
    camera_targets: &Query<Entity, With<CameraTarget>>,
    time: &Res<Time>,
) {
    // Remove old throwing indicator if exists
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove CameraTarget from any existing blocks
    for entity in camera_targets.iter() {
        commands.entity(entity).remove::<CameraTarget>();
    }

    let block_size = 1.0;
    let throwing_cfg = &config.throwing;
    let physics_cfg = &config.physics.block;
    let throw_pos = &config.arena.throwing_line;

    // Use throwing module to calculate velocities
    let throw_params = ThrowParameters::new(
        game_state.throw_angle,
        game_state.throw_rotation,
        game_state.throw_power,
        throwing_cfg.power_multiplier,
    );

    // Random spin ranges (hardcoded, not configurable) - in radians/sec
    // Values give range of -5 to 5 rad/sec on X/Z, -3 to 3 rad/sec on Y
    let spin_range_vec = Vec3::new(10.0, 6.0, 10.0);
    let throw_result = throw_params.calculate(Some(&spin_range_vec));

    // Starting position from config
    let start_pos = Vec3::new(throw_pos.x, throw_pos.y, throw_pos.z);

    // Get player color configuration
    let player_config = player_manager
        .get_player(game_state.current_player)
        .unwrap();

    // Create mesh and material
    let block_mesh = meshes.add(Cuboid::new(block_size, block_size, block_size));
    let block_material = materials.add(StandardMaterial {
        base_color: player_config.color,
        emissive: player_config.emissive,
        // Add slight metallic and reduce roughness for subtle edge highlights
        metallic: 0.1,
        perceptual_roughness: 0.6,
        ..default()
    });

    // Spawn block using bundle
    commands.spawn(PlayerBlockBundle::new(
        block_mesh,
        block_material,
        start_pos,
        throw_result.linear_velocity,
        throw_result.angular_velocity,
        game_state.current_player,
        physics_cfg,
        block_size,
        config.game.fade_in_duration,
        time.elapsed_secs(),
    ));
}
