// Component markers for game entities
use avian3d::prelude::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
#[cfg(any(not(target_arch = "wasm32"), feature = "webgpu"))]
use bevy::post_process::bloom::{Bloom, BloomCompositeMode};
#[cfg(not(target_arch = "wasm32"))]
use bevy::post_process::motion_blur::MotionBlur;
use bevy::prelude::*;

#[derive(Component)]
pub struct Block;

#[derive(Component)]
pub struct TargetBlock;

#[derive(Component)]
pub struct PlayerBlock {
    pub player: usize,
}

#[derive(Component)]
pub struct OutOfBounds; // Marker for blocks that fell off the map

#[derive(Component)]
pub struct ThrowingBlock;

#[derive(Component)]
pub struct ActiveBlock; // Marker for the most recently thrown block (for real-time tracking)

#[derive(Component)]
pub struct CameraTarget; // Marker for the block the camera should follow

#[derive(Component)]
pub struct SettledBlock {
    pub settled_time: f32, // Time when block became settled
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct GameText;

#[derive(Component)]
pub struct Player1ScoreText;

#[derive(Component)]
pub struct Player2ScoreText;

#[derive(Component)]
pub struct FadingOut {
    pub start_time: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct FadingIn {
    pub start_time: f32,
    pub duration: f32,
    pub start_alpha: f32, // Initial alpha value (usually 0.0)
}

#[derive(Component)]
pub struct VictoryAnimation {
    pub start_time: f32,
    pub is_winner: bool, // true if this block belongs to the winner
}

#[derive(Component)]
pub struct MatchVictoryAnimation {
    pub winner_player: usize, // Which player won the match
}

// =============================================================================
// BUNDLES - Component groups for spawning entities
// =============================================================================

/// Bundle for spawning a player's thrown block with all required components
#[derive(Bundle)]
pub struct PlayerBlockBundle {
    // Rendering components
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,

    // Physics components
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub mass: Mass,
    pub linear_velocity: LinearVelocity,
    pub angular_velocity: AngularVelocity,
    pub linear_damping: LinearDamping,
    pub angular_damping: AngularDamping,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub restitution: Restitution,
    pub swept_ccd: SweptCcd,
    pub sleeping_disabled: SleepingDisabled,

    // Game marker components
    pub block: Block,
    pub player_block: PlayerBlock,
    pub active_block: ActiveBlock,
    pub camera_target: CameraTarget,
    pub fading_in: FadingIn,
}

impl PlayerBlockBundle {
    /// Create a new player block bundle with all components configured
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        position: Vec3,
        linear_velocity: Vec3,
        angular_velocity: Vec3,
        player: usize,
        physics: &crate::core::config::BlockPhysicsConfig,
        block_size: f32,
        fade_in_duration: f32,
        current_time: f32,
    ) -> Self {
        Self {
            mesh: Mesh3d(mesh),
            material: MeshMaterial3d(material),
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cuboid(block_size, block_size, block_size),
            mass: Mass(physics.mass),
            linear_velocity: LinearVelocity(linear_velocity),
            angular_velocity: AngularVelocity(angular_velocity),
            linear_damping: LinearDamping(physics.linear_damping),
            angular_damping: AngularDamping(physics.angular_damping),
            gravity_scale: GravityScale(physics.gravity_scale),
            friction: Friction::new(physics.friction),
            restitution: Restitution::new(physics.restitution),
            swept_ccd: SweptCcd::default(),
            sleeping_disabled: SleepingDisabled,
            block: Block,
            player_block: PlayerBlock { player },
            active_block: ActiveBlock,
            camera_target: CameraTarget,
            fading_in: FadingIn {
                start_time: current_time,
                duration: fade_in_duration,
                start_alpha: 0.0,
            },
        }
    }
}

/// Bundle for spawning the target block
#[derive(Bundle)]
pub struct TargetBlockBundle {
    // Rendering components
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,

    // Physics components
    pub rigid_body: RigidBody,
    pub collider: Collider,

    // Game marker components
    pub target_block: TargetBlock,
    pub fading_in: FadingIn,
}

impl TargetBlockBundle {
    /// Create a new target block bundle
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        position: Vec3,
        block_size: f32,
        fade_in_duration: f32,
        current_time: f32,
    ) -> Self {
        Self {
            mesh: Mesh3d(mesh),
            material: MeshMaterial3d(material),
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Static,
            collider: Collider::cuboid(block_size, block_size, block_size),
            target_block: TargetBlock,
            fading_in: FadingIn {
                start_time: current_time,
                duration: fade_in_duration,
                start_alpha: 0.0,
            },
        }
    }
}

/// Bundle for spawning the arena ground
#[derive(Bundle)]
pub struct GroundBundle {
    // Rendering components
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,

    // Physics components
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub friction: Friction,
    pub restitution: Restitution,
}

impl GroundBundle {
    /// Create a new ground bundle
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        position: Vec3,
        dimensions: Vec3,
        physics: &crate::core::config::SurfacePhysicsConfig,
    ) -> Self {
        Self {
            mesh: Mesh3d(mesh),
            material: MeshMaterial3d(material),
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Static,
            collider: Collider::cuboid(dimensions.x, dimensions.y, dimensions.z),
            friction: Friction::new(physics.friction),
            restitution: Restitution::new(physics.restitution),
        }
    }
}

/// Bundle for spawning the invisible safety floor
#[derive(Bundle)]
pub struct SafetyFloorBundle {
    // Rendering components
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,

    // Physics components
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub restitution: Restitution,
}

impl SafetyFloorBundle {
    /// Create a new safety floor bundle
    pub fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        position: Vec3,
        width: f32,
        height: f32,
        depth: f32,
    ) -> Self {
        Self {
            mesh: Mesh3d(mesh),
            material: MeshMaterial3d(material),
            transform: Transform::from_translation(position),
            rigid_body: RigidBody::Static,
            collider: Collider::cuboid(width, height, depth),
            restitution: Restitution::new(0.0), // No bounce on safety floor
        }
    }
}

/// Bundle for spawning the main camera with post-processing effects
#[derive(Bundle)]
pub struct MainCameraBundle {
    pub camera: Camera3d,
    pub transform: Transform,
    pub tonemapping: Tonemapping,
    #[cfg(any(not(target_arch = "wasm32"), feature = "webgpu"))]
    pub bloom: Bloom,
    #[cfg(not(target_arch = "wasm32"))]
    pub motion_blur: MotionBlur,
    pub main_camera: MainCamera,
}

impl MainCameraBundle {
    /// Create a new main camera bundle
    pub fn new(
        position: Vec3,
        look_at: Vec3,
        bloom: &crate::core::config::BloomConfig,
        motion_blur: &crate::core::config::MotionBlurConfig,
    ) -> Self {
        #[cfg(target_arch = "wasm32")]
        let _ = (bloom, motion_blur);
        Self {
            camera: Camera3d::default(),
            transform: Transform::from_translation(position).looking_at(look_at, Vec3::Y),
            tonemapping: Tonemapping::TonyMcMapface,
            #[cfg(any(not(target_arch = "wasm32"), feature = "webgpu"))]
            bloom: Bloom {
                intensity: bloom.intensity,
                low_frequency_boost: bloom.low_frequency_boost,
                low_frequency_boost_curvature: bloom.low_frequency_boost_curvature,
                high_pass_frequency: bloom.high_pass_frequency,
                composite_mode: BloomCompositeMode::Additive,
                ..default()
            },
            #[cfg(not(target_arch = "wasm32"))]
            motion_blur: MotionBlur {
                shutter_angle: motion_blur.shutter_angle,
                samples: motion_blur.samples,
            },
            main_camera: MainCamera,
        }
    }
}
