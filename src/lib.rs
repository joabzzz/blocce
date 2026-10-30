// BLOCCE - Physics-based competitive block throwing game
//
// This library exposes the core game systems and can be used
// for future features like AI opponents, replays, or testing.

pub mod core;
pub mod gameplay;
pub mod physics;
pub mod rendering;
pub mod ui;

// Re-export commonly used types for external use
pub use core::{
    ActiveBlock, Block, CameraTarget, GameConfiguration, GamePhase, GameState, PlayerBlock,
    PlayerManager, TargetBlock,
};
pub use gameplay::setup_arena;
pub use physics::{calculate_throw_velocity, ThrowParameters};
pub use rendering::setup_camera;
