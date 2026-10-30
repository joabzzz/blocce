// Arena and course setup
use bevy::prelude::*;
use rand::Rng;

use crate::core::{GameConfiguration, GroundBundle, SafetyFloorBundle, TargetBlockBundle};

/// Setup the arena with ground and target using configuration
pub fn setup_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameConfiguration>,
    time: Res<Time>,
) {
    spawn_ground(&mut commands, &mut meshes, &mut materials, &config);
    spawn_safety_floor(&mut commands, &mut meshes, &mut materials, &config);
    spawn_target(&mut commands, &mut meshes, &mut materials, &config, &time);
}

/// Spawn the ground plane
fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &GameConfiguration,
) {
    let ground = &config.arena.ground;
    let physics = &config.physics.ground;

    println!("=== ARENA SETUP ===");
    println!(
        "Ground dimensions: {}x{}x{}",
        ground.width, ground.height, ground.depth
    );
    println!(
        "Ground position: y={} (top surface at y={})",
        ground.y_position,
        ground.y_position + ground.height / 2.0
    );

    // Create mesh and material for ground
    let ground_mesh = meshes.add(Cuboid::new(ground.width, ground.height, ground.depth));
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.15),
        perceptual_roughness: 0.95,
        ..default()
    });

    // Spawn ground using bundle
    commands.spawn(GroundBundle::new(
        ground_mesh,
        ground_material,
        Vec3::new(0.0, ground.y_position, 0.0),
        Vec3::new(ground.width, ground.height, ground.depth),
        physics,
    ));

    println!(
        "Course boundaries: X: ±{}, Z: ±{}",
        config.arena.boundaries.x_max, config.arena.boundaries.z_max
    );
}

/// Spawn an invisible safety floor to catch falling blocks
fn spawn_safety_floor(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &GameConfiguration,
) {
    let safety = &config.arena.safety_floor;

    println!("Safety floor at y={}", safety.y_position);

    // Create mesh and material for safety floor (invisible)
    let safety_mesh = meshes.add(Cuboid::new(safety.width, safety.height, safety.depth));
    let safety_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.0), // Fully transparent
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Spawn safety floor using bundle
    commands.spawn(SafetyFloorBundle::new(
        safety_mesh,
        safety_material,
        Vec3::new(0.0, safety.y_position, 0.0),
        safety.width,
        safety.height,
        safety.depth,
    ));
}

/// Spawn the target block with random X and Z position
pub fn spawn_target(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &GameConfiguration,
    time: &Res<Time>,
) {
    let target_cfg = &config.arena.target;
    let emissive_cfg = &config.rendering.target_emissive;

    let mut rng = rand::thread_rng();
    let target_x = rng.gen_range(target_cfg.random_x_min..target_cfg.random_x_max);
    let target_z = rng.gen_range(target_cfg.random_z_min..target_cfg.random_z_max);

    println!(
        "Target position: ({}, {}, {})",
        target_x, target_cfg.y_position, target_z
    );

    let light_config = &config.lighting.block_light;

    // Create mesh and material for target block
    let target_mesh = meshes.add(Cuboid::new(
        target_cfg.size,
        target_cfg.size,
        target_cfg.size,
    ));
    let target_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        emissive: emissive_cfg.to_linear(),
        ..default()
    });

    // Spawn target using bundle
    commands
        .spawn(TargetBlockBundle::new(
            target_mesh,
            target_material,
            Vec3::new(target_x, target_cfg.y_position, target_z),
            target_cfg.size,
            config.game.fade_in_duration,
            time.elapsed_secs(),
        ))
        .with_children(|parent| {
            // Add a small point light for glimmer effect
            parent.spawn(PointLight {
                color: Color::srgb(1.0, 1.0, 1.0),
                intensity: light_config.intensity * 0.3, // 30% of block light intensity
                radius: light_config.radius * 0.5,
                range: light_config.range * 0.5,
                shadows_enabled: false, // No shadows for target light
                ..default()
            });
        });

    println!("=== ARENA SETUP COMPLETE ===\n");
}

/// Setup cinematic lighting using configuration
pub fn setup_lights(mut commands: Commands, config: Res<GameConfiguration>) {
    let dir_light = &config.lighting.directional;
    let ambient = &config.lighting.ambient;

    // Directional light (sun/moon)
    commands.spawn((
        DirectionalLight {
            color: dir_light.color(),
            illuminance: dir_light.illuminance,
            shadows_enabled: dir_light.shadows_enabled,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            dir_light.pitch,
            dir_light.yaw,
            0.0,
        )),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: ambient.color(),
        brightness: ambient.brightness,
        affects_lightmapped_meshes: false,
    });
}
