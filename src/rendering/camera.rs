// Camera setup and movement
use bevy::prelude::*;

use crate::core::{CameraTarget, GameConfiguration, GameState, MainCamera, MainCameraBundle};

/// Setup the main camera with bloom and motion blur effects
pub fn setup_camera(mut commands: Commands, config: Res<GameConfiguration>) {
    let initial = &config.camera.initial;
    let bloom_cfg = &config.rendering.bloom;
    let motion_blur_cfg = &config.rendering.motion_blur;

    // Spawn camera using bundle
    commands.spawn(MainCameraBundle::new(
        Vec3::new(initial.x, initial.y, initial.z),
        Vec3::new(initial.look_at_x, initial.look_at_y, initial.look_at_z),
        bloom_cfg,
        motion_blur_cfg,
    ));
}

/// Move camera based on player input and game state
pub fn move_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_state: Res<GameState>,
    config: Res<GameConfiguration>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    target_block_query: Query<&Transform, (With<CameraTarget>, Without<MainCamera>)>,
    time: Res<Time>,
) {
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        let manual = &config.camera.manual;
        let orbit = &config.camera.orbit;

        // Check if we should follow a block
        if let Ok(target_transform) = target_block_query.single() {
            // Follow the thrown block
            let block_pos = target_transform.translation;

            // Position camera behind and above the block
            let follow_offset = Vec3::new(0.0, 3.0, -5.0); // Above and behind
            let target_pos = block_pos + follow_offset;

            // Smoothly follow the block
            camera_transform.translation = camera_transform
                .translation
                .lerp(target_pos, 5.0 * time.delta_secs());

            // Look at the block
            camera_transform.look_at(block_pos, Vec3::Y);
        } else {
            // No target block - use default camera behavior

            // Get forward vector from the transform
            let forward_vec = camera_transform.forward().as_vec3();
            let forward = Vec3::new(forward_vec.x, 0.0, forward_vec.z).normalize_or_zero();

            // Manual camera movement - forward/back and up/down only
            if keyboard.pressed(KeyCode::ArrowUp) {
                camera_transform.translation += forward * manual.move_speed * time.delta_secs();
            }
            if keyboard.pressed(KeyCode::ArrowDown) {
                camera_transform.translation -= forward * manual.move_speed * time.delta_secs();
            }
            if keyboard.pressed(KeyCode::KeyW) {
                camera_transform.translation.y += manual.height_adjust_speed * time.delta_secs();
            }
            if keyboard.pressed(KeyCode::KeyS) {
                camera_transform.translation.y -= manual.height_adjust_speed * time.delta_secs();
            }

            // Camera rotates around orbital center point
            let center_point = Vec3::new(orbit.center_x, orbit.center_y, orbit.center_z);
            let rotation_rad = game_state.throw_rotation.to_radians();

            // Rotate offset around center point
            let offset_x = -orbit.distance * rotation_rad.sin();
            let offset_z = -orbit.distance * rotation_rad.cos();

            let target_camera_pos = center_point + Vec3::new(offset_x, orbit.height, offset_z);

            // Smoothly interpolate camera to target position
            camera_transform.translation = camera_transform.translation.lerp(
                target_camera_pos,
                orbit.interpolation_speed * time.delta_secs(),
            );

            // Look at a point forward from center in the throw direction
            let look_target = center_point
                + Vec3::new(
                    orbit.look_distance * rotation_rad.sin(),
                    0.0,
                    orbit.look_distance * rotation_rad.cos(),
                );

            camera_transform.look_at(look_target, Vec3::Y);
        }
    }
}
