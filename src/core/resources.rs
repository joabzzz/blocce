// Game state resources
// Configuration is now in config.rs
use super::config::GameConfiguration;
use bevy::prelude::*;

/// Game phase tracking - explicit state machine for round progression
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Playing,          // Players are throwing blocks (3 shots each)
    BlocksSettling,   // All blocks thrown, waiting for physics to settle
    VictoryAnimation, // Blocks settled, showing supernova effect
    FadingOut,        // Victory animation complete, fading out blocks
    MatchComplete,    // Someone won the best-of-7 match
}

/// Main game state resource
#[derive(Resource)]
pub struct GameState {
    pub current_player: usize,
    pub starting_player: usize, // Which player starts the current round (alternates for fairness)
    pub blocks_thrown: [usize; 2],
    pub scores: [f32; 2],       // Current round distances
    pub round_wins: [usize; 2], // Best of 7 score
    pub throw_power: f32,
    pub throw_angle: f32,
    pub throw_rotation: f32,
    pub all_blocks_settled: bool,
    pub phase: GamePhase,
    pub phase_start_time: f32,            // Time when current phase started
    pub settling_start_time: Option<f32>, // Time when blocks first appeared settled
    pub round_winner: Option<usize>,      // Winner of current round (0, 1, or None for tie)
}

impl GameState {
    /// Create from configuration
    pub fn from_config(config: &GameConfiguration) -> Self {
        Self {
            current_player: 0,
            starting_player: 0, // Player 1 starts first round
            blocks_thrown: [0, 0],
            scores: [0.0, 0.0],
            round_wins: [0, 0],
            throw_power: config.throwing.min_power,
            throw_angle: config.throwing.default_angle,
            throw_rotation: config.throwing.default_rotation,
            all_blocks_settled: true,
            phase: GamePhase::Playing,
            phase_start_time: 0.0,
            settling_start_time: None,
            round_winner: None,
        }
    }

    /// Transition to a new game phase
    pub fn transition_to_phase(&mut self, new_phase: GamePhase, current_time: f32) {
        if self.phase != new_phase {
            println!(
                "\n=== PHASE TRANSITION: {:?} -> {:?} ===",
                self.phase, new_phase
            );
            self.phase = new_phase;
            self.phase_start_time = current_time;
        }
    }

    /// Reset state for a new round
    pub fn start_new_round(&mut self, config: &GameConfiguration) {
        println!("\n=== STARTING NEW ROUND ===");
        println!(
            "Match Score - P1: {} | P2: {}\n",
            self.round_wins[0], self.round_wins[1]
        );

        // Alternate starting player for fairness
        self.starting_player = 1 - self.starting_player; // 0 -> 1, 1 -> 0
        self.current_player = self.starting_player;

        println!("Player {} starts this round\n", self.starting_player + 1);

        self.blocks_thrown = [0, 0];
        self.scores = [0.0, 0.0];
        self.throw_power = config.throwing.min_power;
        self.throw_rotation = config.throwing.default_rotation;
        self.all_blocks_settled = true;
        self.settling_start_time = None;
        self.round_winner = None;
        self.phase = GamePhase::Playing;
    }
}
