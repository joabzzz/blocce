// Game logic, rules, and arena setup

pub mod ai;
pub mod animation;
pub mod arena;
pub mod game;
pub mod input;
pub mod physics;
pub mod rounds;

// Re-export public API
// (game module re-exports animation, input, physics, and rounds)
pub use ai::*;
pub use arena::*;
pub use game::*;
