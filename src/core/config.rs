use bevy::prelude::*;
use serde::Deserialize;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;

/// Root configuration structure loaded from config.yaml
#[derive(Debug, Clone, Deserialize, Resource)]
#[allow(dead_code)]
pub struct GameConfiguration {
    pub game: GameRulesConfig,
    pub physics: PhysicsConfig,
    pub throwing: ThrowingConfig,
    pub arena: ArenaConfig,
    pub lighting: LightingConfig,
    pub rendering: RenderingConfig,
    pub camera: CameraConfig,
    pub players: Vec<PlayerConfigData>,
    pub animation: AnimationConfig,
    pub ui: UIConfig,
    pub input: InputConfig,
}

impl GameConfiguration {
    /// Load configuration from config.yaml file
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Result<Self, String> {
        let config_path = "config.yaml";
        let config_str = fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read {}: {}", config_path, e))?;

        serde_yaml::from_str(&config_str)
            .map_err(|e| format!("Failed to parse {}: {}", config_path, e))
    }

    /// Embed the same configuration used by native builds in the browser binary.
    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Result<Self, String> {
        serde_yaml::from_str(include_str!("../../config.yaml"))
            .map_err(|e| format!("Failed to parse embedded config.yaml: {}", e))
    }

    /// Load configuration with fallback to default if file doesn't exist
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_else(|e| {
            eprintln!("Warning: {}", e);
            eprintln!("Using default configuration");
            Self::default()
        })
    }
}

impl Default for GameConfiguration {
    fn default() -> Self {
        // Fallback defaults matching the config.yaml values
        Self {
            game: GameRulesConfig::default(),
            physics: PhysicsConfig::default(),
            throwing: ThrowingConfig::default(),
            arena: ArenaConfig::default(),
            lighting: LightingConfig::default(),
            rendering: RenderingConfig::default(),
            camera: CameraConfig::default(),
            players: vec![
                PlayerConfigData {
                    name: "Player 1".to_string(),
                    color: ColorConfig {
                        r: 1.0,
                        g: 0.3,
                        b: 0.0,
                    },
                    emissive: ColorConfig {
                        r: 1.75,
                        g: 0.525,
                        b: 0.0,
                    },
                },
                PlayerConfigData {
                    name: "Player 2".to_string(),
                    color: ColorConfig {
                        r: 0.0,
                        g: 0.4,
                        b: 1.0,
                    },
                    emissive: ColorConfig {
                        r: 0.0,
                        g: 0.7,
                        b: 2.1875,
                    },
                },
            ],
            animation: AnimationConfig::default(),
            ui: UIConfig::default(),
            input: InputConfig::default(),
        }
    }
}

// ============================================================================
// Game Rules Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct GameRulesConfig {
    pub rounds_to_win: usize,
    pub blocks_per_player: usize,
    pub settle_duration: f32,
    pub victory_display_duration: f32,
    pub fade_in_duration: f32,
    pub fade_out_duration: f32,
    pub match_complete_delay: f32,
}

impl Default for GameRulesConfig {
    fn default() -> Self {
        Self {
            rounds_to_win: 4,
            blocks_per_player: 3,
            settle_duration: 1.0,
            victory_display_duration: 5.0,
            fade_in_duration: 0.4,
            fade_out_duration: 1.5,
            match_complete_delay: 3.0,
        }
    }
}

// ============================================================================
// Physics Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct PhysicsConfig {
    pub gravity: f32,
    pub substeps: u32,
    pub max_overlap_solve_speed: f32,
    pub restitution_iterations: u32,
    pub block: BlockPhysicsConfig,
    pub ground: SurfacePhysicsConfig,
    pub settlement: SettlementConfig,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: -9.81,
            substeps: 12,
            max_overlap_solve_speed: 5.0,
            restitution_iterations: 4,
            block: BlockPhysicsConfig::default(),
            ground: SurfacePhysicsConfig::default(),
            settlement: SettlementConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct BlockPhysicsConfig {
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub gravity_scale: f32,
    pub ccd_enabled: bool,
    pub disable_sleeping: bool,
}

impl Default for BlockPhysicsConfig {
    fn default() -> Self {
        Self {
            mass: 5.0,
            friction: 0.7,
            restitution: 0.5,
            linear_damping: 0.2,
            angular_damping: 0.5,
            gravity_scale: 1.0,
            ccd_enabled: true,
            disable_sleeping: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SurfacePhysicsConfig {
    pub friction: f32,
    pub restitution: f32,
}

impl Default for SurfacePhysicsConfig {
    fn default() -> Self {
        Self {
            friction: 0.8,
            restitution: 0.2,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettlementConfig {
    pub velocity_threshold: f32,
    pub angular_velocity_threshold: f32,
}

impl Default for SettlementConfig {
    fn default() -> Self {
        Self {
            velocity_threshold: 0.3,
            angular_velocity_threshold: 0.3,
        }
    }
}

// ============================================================================
// Throwing Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ThrowingConfig {
    pub power_multiplier: f32,
    pub charge_rate: f32,
    pub min_power: f32,
    pub max_power: f32,
    pub default_angle: f32,
    pub min_angle: f32,
    pub max_angle: f32,
    pub rotation_speed: f32,
    pub min_rotation: f32,
    pub max_rotation: f32,
    pub default_rotation: f32,
}

impl Default for ThrowingConfig {
    fn default() -> Self {
        Self {
            power_multiplier: 20.0,
            charge_rate: 0.8,
            min_power: 0.5,
            max_power: 1.0,
            default_angle: 45.0,
            min_angle: 15.0,
            max_angle: 75.0,
            rotation_speed: 90.0,
            min_rotation: -45.0,
            max_rotation: 45.0,
            default_rotation: 0.0,
        }
    }
}

// ============================================================================
// Arena Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ArenaConfig {
    pub ground: GroundConfig,
    pub safety_floor: SafetyFloorConfig,
    pub target: TargetConfig,
    pub boundaries: BoundaryConfig,
    pub throwing_line: PositionConfig,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            ground: GroundConfig::default(),
            safety_floor: SafetyFloorConfig::default(),
            target: TargetConfig::default(),
            boundaries: BoundaryConfig::default(),
            throwing_line: PositionConfig {
                x: 0.0,
                y: 2.0,
                z: -15.0,
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GroundConfig {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub y_position: f32,
}

impl Default for GroundConfig {
    fn default() -> Self {
        Self {
            width: 30.0,
            height: 2.0,
            depth: 40.0,
            y_position: -1.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SafetyFloorConfig {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub y_position: f32,
}

impl Default for SafetyFloorConfig {
    fn default() -> Self {
        Self {
            width: 200.0,
            height: 1.0,
            depth: 200.0,
            y_position: -25.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargetConfig {
    pub size: f32,
    pub y_position: f32,
    pub random_x_min: f32,
    pub random_x_max: f32,
    pub random_z_min: f32,
    pub random_z_max: f32,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            size: 0.3,
            y_position: 0.15,
            random_x_min: -10.0,
            random_x_max: 10.0,
            random_z_min: 5.0,
            random_z_max: 15.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoundaryConfig {
    pub x_max: f32,
    pub z_max: f32,
    pub y_despawn_immediate: f32,
    pub y_despawn_cleanup: f32,
    pub oob_despawn_velocity: f32,
}

impl Default for BoundaryConfig {
    fn default() -> Self {
        Self {
            x_max: 15.0,
            z_max: 20.0,
            y_despawn_immediate: -5.0,
            y_despawn_cleanup: -20.0,
            oob_despawn_velocity: 0.1,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PositionConfig {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// ============================================================================
// Lighting Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LightingConfig {
    pub directional: DirectionalLightConfig,
    pub ambient: AmbientLightConfig,
    pub block_light: BlockLightConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectionalLightConfig {
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub illuminance: f32,
    pub shadows_enabled: bool,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for DirectionalLightConfig {
    fn default() -> Self {
        Self {
            color_r: 0.15,
            color_g: 0.2,
            color_b: 0.3,
            illuminance: 200.0,
            shadows_enabled: true,
            pitch: -1.0,
            yaw: 0.5,
        }
    }
}

impl DirectionalLightConfig {
    pub fn color(&self) -> Color {
        Color::srgb(self.color_r, self.color_g, self.color_b)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AmbientLightConfig {
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub brightness: f32,
}

impl Default for AmbientLightConfig {
    fn default() -> Self {
        Self {
            color_r: 0.01,
            color_g: 0.02,
            color_b: 0.05,
            brightness: 10.0,
        }
    }
}

impl AmbientLightConfig {
    pub fn color(&self) -> Color {
        Color::srgb(self.color_r, self.color_g, self.color_b)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockLightConfig {
    pub intensity: f32,
    pub radius: f32,
    pub range: f32,
    pub shadows_enabled: bool,
}

impl Default for BlockLightConfig {
    fn default() -> Self {
        Self {
            intensity: 300000.0,
            radius: 5.0,
            range: 8.0,
            shadows_enabled: true,
        }
    }
}

// ============================================================================
// Rendering Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct RenderingConfig {
    pub clear_color: ColorConfig,
    pub bloom: BloomConfig,
    pub target_emissive: ColorConfig,
    pub motion_blur: MotionBlurConfig,
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            clear_color: ColorConfig {
                r: 0.01,
                g: 0.02,
                b: 0.05,
            },
            bloom: BloomConfig::default(),
            target_emissive: ColorConfig {
                r: 1.875,
                g: 1.875,
                b: 1.875,
            },
            motion_blur: MotionBlurConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct BloomConfig {
    pub intensity: f32,
    pub low_frequency_boost: f32,
    pub low_frequency_boost_curvature: f32,
    pub high_pass_frequency: f32,
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            intensity: 0.4,
            low_frequency_boost: 0.8,
            low_frequency_boost_curvature: 0.95,
            high_pass_frequency: 1.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct MotionBlurConfig {
    pub shutter_angle: f32,
    pub samples: u32,
}

impl Default for MotionBlurConfig {
    fn default() -> Self {
        Self {
            shutter_angle: 0.8,
            samples: 6,
        }
    }
}

// ============================================================================
// Camera Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CameraConfig {
    pub initial: InitialCameraConfig,
    pub orbit: OrbitCameraConfig,
    pub manual: ManualCameraConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InitialCameraConfig {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub look_at_x: f32,
    pub look_at_y: f32,
    pub look_at_z: f32,
}

impl Default for InitialCameraConfig {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 15.0,
            z: -18.0,
            look_at_x: 0.0,
            look_at_y: 0.0,
            look_at_z: 0.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrbitCameraConfig {
    pub center_x: f32,
    pub center_y: f32,
    pub center_z: f32,
    pub distance: f32,
    pub height: f32,
    pub look_distance: f32,
    pub interpolation_speed: f32,
}

impl Default for OrbitCameraConfig {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            center_z: -15.0,
            distance: 8.0,
            height: 12.0,
            look_distance: 15.0,
            interpolation_speed: 5.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManualCameraConfig {
    pub move_speed: f32,
    pub height_adjust_speed: f32,
}

impl Default for ManualCameraConfig {
    fn default() -> Self {
        Self {
            move_speed: 10.0,
            height_adjust_speed: 5.0,
        }
    }
}

// ============================================================================
// Player Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerConfigData {
    pub name: String,
    pub color: ColorConfig,
    pub emissive: ColorConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColorConfig {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl ColorConfig {
    pub fn to_srgb(&self) -> Color {
        Color::srgb(self.r, self.g, self.b)
    }

    pub fn to_linear(&self) -> LinearRgba {
        LinearRgba::rgb(self.r, self.g, self.b)
    }
}

// ============================================================================
// Animation Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct AnimationConfig {
    pub victory: VictoryAnimationConfig,
    pub fade: FadeAnimationConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VictoryAnimationConfig {
    pub winner_brightness_multiplier: f32,
    pub loser_brightness_multiplier: f32,
}

impl Default for VictoryAnimationConfig {
    fn default() -> Self {
        Self {
            winner_brightness_multiplier: 2.0,
            loser_brightness_multiplier: 0.01,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct FadeAnimationConfig {
    pub start_alpha: f32,
    pub end_alpha: f32,
}

impl Default for FadeAnimationConfig {
    fn default() -> Self {
        Self {
            start_alpha: 1.0,
            end_alpha: 0.0,
        }
    }
}

// ============================================================================
// UI Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct UIConfig {
    pub score_text: TextStyleConfig,
    pub game_text: TextStyleConfig,
    pub match_winner_text: TextStyleConfig,
    pub controls_text: TextStyleConfig,
    pub score_margin: f32,
    pub controls_bottom_margin: f32,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            score_text: TextStyleConfig {
                size: 40.0,
                color_r: 1.0,
                color_g: 1.0,
                color_b: 1.0,
            },
            game_text: TextStyleConfig {
                size: 24.0,
                color_r: 1.0,
                color_g: 1.0,
                color_b: 1.0,
            },
            match_winner_text: TextStyleConfig {
                size: 72.0,
                color_r: 1.0,
                color_g: 1.0,
                color_b: 1.0,
            },
            controls_text: TextStyleConfig {
                size: 16.0,
                color_r: 0.7,
                color_g: 0.7,
                color_b: 0.7,
            },
            score_margin: 20.0,
            controls_bottom_margin: 20.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextStyleConfig {
    pub size: f32,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
}

impl TextStyleConfig {
    pub fn color(&self) -> Color {
        Color::srgb(self.color_r, self.color_g, self.color_b)
    }
}

// ============================================================================
// Input Configuration
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct InputConfig {
    pub aim_left: String,
    pub aim_right: String,
    pub throw: String,
    pub camera_forward: String,
    pub camera_back: String,
    pub camera_up: String,
    pub camera_down: String,
    pub spin_left: String,
    pub spin_right: String,
    pub pitch_up: String,
    pub pitch_down: String,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            aim_left: "KeyA".to_string(),
            aim_right: "KeyD".to_string(),
            throw: "Space".to_string(),
            camera_forward: "ArrowUp".to_string(),
            camera_back: "ArrowDown".to_string(),
            camera_up: "KeyW".to_string(),
            camera_down: "KeyS".to_string(),
            spin_left: "KeyQ".to_string(),
            spin_right: "KeyE".to_string(),
            pitch_up: "KeyW".to_string(),
            pitch_down: "KeyS".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GameConfiguration;

    #[test]
    fn checked_in_configuration_is_valid() {
        let config: GameConfiguration = serde_yaml::from_str(include_str!("../../config.yaml"))
            .expect("config.yaml should parse for the browser build");
        assert!(config.game.rounds_to_win > 0);
        assert!(config.game.blocks_per_player > 0);
    }
}
