// Game logic - split into focused submodules
//
// This file re-exports functions from submodules for backward compatibility.
// Functions are organized into:
// - input: Player input and block throwing
// - physics: Distance tracking, out-of-bounds, camera targeting
// - rounds: Round progression, scoring, phase management
// - animation: Victory effects, fade transitions, match completion

// Re-export all public functions from submodules
pub use super::animation::*;
pub use super::input::*;
pub use super::physics::*;
pub use super::rounds::*;
