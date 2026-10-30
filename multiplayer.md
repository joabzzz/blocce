## User

Make no changes, but I want to explore what a game server would look like for this game? What things would happen server side vs client? Since it's physics it sounds like a lot is predictable but collisions need to be managed

## Assistant

I'll explore the codebase first to understand the game's architecture and physics, then discuss what a server architecture would look like for this game.

**Tool Call: Explore game architecture and physics**
Status: Completed

Now I have enough information. Let me create a comprehensive summary:

## BLOCCE Game Architecture Analysis

Based on my thorough exploration of the codebase, here's a detailed summary of the BLOCCE game's architecture:

---

### 1. GAME TYPE & CORE MECHANICS

**Game Type:** Physics-based competitive block throwing game (digital bocce ball)
- **Format:** Best-of-7 match system (first to 4 rounds wins)
- **Players:** 2-player turn-based competition
- **Per Round:** Each player throws 3 blocks toward a randomly positioned target
- **Winning Condition:** Closest block to the target wins the round

**File Location:** `/Users/sandro/Development/blocce/src/main.rs`, `/Users/sandro/Development/blocce/README.md`

---

### 2. PHYSICS IMPLEMENTATION

**Physics Engine:** Avian3D 0.4 (ECS-based physics for Bevy)

**Key Physics Configuration:**
```rust
// Location: /Users/sandro/Development/blocce/src/core/config.rs

Gravity: -9.81 m/s²
Substeps: 12 per frame (high stability for accurate collision detection)
Max Overlap Solve Speed: 5.0
Restitution Iterations: 4

Block Properties:
- Mass: 5.0 kg
- Friction: 0.7
- Restitution (bounciness): 0.5
- Linear Damping (air resistance): 0.2
- Angular Damping: 0.5
- Gravity Scale: 1.0
- CCD Enabled: true (Continuous Collision Detection for fast-moving objects)
- Sleeping Disabled: true (blocks never "sleep" during gameplay)

Ground Properties:
- Friction: 0.8
- Restitution: 0.2
```

**Throw Velocity Calculation:**
```rust
// Location: /Users/sandro/Development/blocce/src/physics/throwing.rs

pub fn calculate_throw_velocity(
    angle: f32,      // Vertical pitch (45° default)
    rotation: f32,   // Horizontal yaw (-45° to +45°)
    power: f32,      // 0.5 to 1.0 normalized
    power_multiplier: f32 // 20.0 default
) -> Vec3

Velocity Components:
- X = cos(angle) * velocity_magnitude * sin(rotation)
- Y = sin(angle) * velocity_magnitude
- Z = cos(angle) * velocity_magnitude * cos(rotation)
```

**Random Spin Application:**
- Applied to every thrown block for realism
- Range: ±5 rad/sec on X/Z axes, ±3 rad/sec on Y axis
- Uses `calculate_random_spin()` function

**Settlement Detection:**
```rust
Blocks considered "settled" when:
- Linear velocity < 0.3 m/s AND
- Angular velocity < 0.3 rad/sec AND
- Held in settled state for 1.0 second
```

**File Locations:**
- `/Users/sandro/Development/blocce/src/physics/throwing.rs`
- `/Users/sandro/Development/blocce/tests/physics_tests.rs` (comprehensive collision tests)

---

### 3. GAME STATE MANAGEMENT

**State Machine Architecture:**

**Application States** (Top-level app state)
```rust
// Location: /Users/sandro/Development/blocce/src/core/states.rs

enum AppState {
    Loading,           // Font loading
    StartMenu,         // Main menu
    PlayerSelection,   // Choose 1P vs 1P, 1P vs CPU, etc.
    Countdown,         // 2-second countdown before match
    InGame,            // Active gameplay
    PauseMenu,         // Pause during InGame
}
```

**Game Phase State** (Micro-level game progression)
```rust
// Location: /Users/sandro/Development/blocce/src/core/resources.rs

enum GamePhase {
    Playing,           // Players actively throwing blocks
    BlocksSettling,    // All blocks thrown, waiting for physics settle
    VictoryAnimation,  // Displaying supernova effect on winner block
    FadingOut,         // Fading out blocks before next round
    MatchComplete,     // Match winner determined (best-of-7 complete)
}
```

**Main Game State Resource:**
```rust
pub struct GameState {
    pub current_player: usize,              // 0 or 1
    pub starting_player: usize,             // Alternates each round
    pub blocks_thrown: [usize; 2],          // Per-player block count
    pub scores: [f32; 2],                   // Current round distances to target
    pub round_wins: [usize; 2],             // Best-of-7 match score
    pub throw_power: f32,                   // 0.5 to 1.0 (charging state)
    pub throw_angle: f32,                   // 45° (fixed, not adjustable)
    pub throw_rotation: f32,                // -45° to +45° (left/right aim)
    pub all_blocks_settled: bool,
    pub phase: GamePhase,
    pub phase_start_time: f32,
    pub settling_start_time: Option<f32>,   // Tracks settlement duration
    pub round_winner: Option<usize>,        // 0, 1, or None (tie)
}
```

**State Transitions:**
```
Playing → (all blocks thrown) → BlocksSettling
         ↓
         (blocks settle after 1 sec)
         ↓
BlocksSettling → (distance calculated) → VictoryAnimation
                 ↓
                 (5 sec supernova effect)
                 ↓
VictoryAnimation → FadingOut
                 ↓
                 (1.5 sec fade)
                 ↓
FadingOut → (round complete) → Playing (next round)
         OR
MatchComplete (after 4 round wins)
```

**File Locations:**
- `/Users/sandro/Development/blocce/src/core/resources.rs`
- `/Users/sandro/Development/blocce/src/core/states.rs`
- `/Users/sandro/Development/blocce/src/gameplay/game.rs` (state transition logic)

---

### 4. INPUT HANDLING & PLAYER ACTIONS

**Input System:**
```rust
// Location: /Users/sandro/Development/blocce/src/gameplay/game.rs

pub fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, ...)

Key Bindings:
┌─────────────────────────────────────────┐
│ Action         │ Key(s)                 │
├─────────────────────────────────────────┤
│ Aim Left       │ A (hold to rotate)    │
│ Aim Right      │ D (hold to rotate)    │
│ Throw/Charge   │ Space (hold & release)│
│ Camera Forward │ ↑ (Arrow Up)          │
│ Camera Back    │ ↓ (Arrow Down)        │
│ Camera Up      │ W                      │
│ Camera Down    │ S                      │
│ Pause          │ P                      │
│ Menu Confirm   │ Space (in menus)       │
└─────────────────────────────────────────┘

Throwing Mechanics:
1. Power Charging
   - Hold Space to charge (0.5 to 1.0 power)
   - Charge Rate: 0.8 power/second
   - Takes ~1.25 seconds to reach max power
   
2. Aim Rotation
   - A/D keys adjust throw_rotation
   - Range: -45° to +45° (90° total range)
   - Rotation Speed: 90°/second
   - Vertical angle: Fixed at 45°

3. Block Spawning
   - Spawn position: (0, 2.0, -15.0)
   - Size: 1.0 × 1.0 × 1.0 meter cube
   - Applied velocity + random spin
   - Marked with ActiveBlock, CameraTarget components
```

**Player Actions - Turn Flow:**
```
Player 0 Turn → Throw Block 1
↓
(Blocks settle check)
↓
Player 1 Turn → Throw Block 1
↓
Player 0 Turn → Throw Block 2
↓
Player 1 Turn → Throw Block 2
↓
Player 0 Turn → Throw Block 3
↓
Player 1 Turn → Throw Block 3
↓
All Blocks Settled → Round Complete → Distance Calculation
```

**File Location:**
- `/Users/sandro/Development/blocce/src/gameplay/game.rs` (handle_input function)

---

### 5. NETWORKING & MULTIPLAYER

**Current Status: NO NETWORKING IMPLEMENTATION**

**Dependencies:**
```toml
// Location: /Users/sandro/Development/blocce/Cargo.toml

[dependencies]
bevy = "0.17.2"           # Game engine
avian3d = "0.4"          # Physics
rand = "0.8"             # Random number generation
serde = "1.0"            # Serialization
serde_yaml = "0.9"       # Config parsing

# NO networking dependencies currently
```

**Implications for Network Architecture:**

The codebase is **completely local single-client**. For multiplayer networking, you would need to:

1. **Add Networking Crate:** Consider `bevy_renet`, `bevy_quinnet`, or raw `tokio` + custom messages
2. **Separate Concerns:**
   - Physics simulation (deterministic, can run client-side or server-side)
   - Game state synchronization
   - Input replication
   - State reconciliation

3. **Key Challenges for Networked BLOCCE:**
   - **Physics Determinism:** Avian3D is deterministic with fixed substeps (12), allowing for lockstep or server-authoritative physics
   - **Turn-Based Coordination:** Current sequential turn system works well for network models
   - **Block Spawning:** Needs server authority to prevent cheating on velocity
   - **Settlement Detection:** Could run on server or client with verification

---

### 6. COLLISION DETECTION & RESOLUTION

**Collision System:**
```rust
// Using Avian3D's built-in collision detection
// Engine: Parry (default for Avian3D)

Collider Types Used:
- Cuboid colliders for all blocks (1×1×1)
- Cuboid for ground plane (30×2×40)
- Cuboid for target (0.3×0.3×0.3)
- Cuboid for safety floor (200×1×200)

Collision Layers (though not explicitly used in current code):
- Ground (static)
- Block (dynamic)
- Target (static)

Physics Resolution:
- Engine: Avian3D with 12 substeps per frame
- Contact solving iterations: 4
- Max overlap solve speed: 5.0 m/s
```

**Collision Testing:**
Tests in `/Users/sandro/Development/blocce/tests/physics_tests.rs` verify:
1. **Block-to-Ground Collision:** Blocks rest on ground without sinking
2. **Block-to-Block Collision:** Stacking behavior works correctly
3. **Block-to-Wall Collision:** Prevents blocks from passing through boundaries

**Out-of-Bounds Detection:**
```rust
// Location: /Users/sandro/Development/blocce/src/gameplay/game.rs

Boundary Checks:
- X boundaries: ±15 meters
- Z boundaries: ±20 meters
- Y despawn immediate: -5 meters
- Y despawn cleanup: -20 meters (safety floor)

Marked with OutOfBounds component, removed from scoring
```

---

### 7. DISTANCE TRACKING & SCORING

**Real-Time Distance Calculation:**
```rust
// Location: /Users/sandro/Development/blocce/src/gameplay/game.rs

pub fn update_distances(
    player_blocks: Query<(&Transform, &PlayerBlock)>,
    target_query: Query<&Transform, With<TargetBlock>>,
) {
    // Continuously tracks 3D Euclidean distance:
    distance = sqrt((block.x - target.x)² + (block.y - target.y)² + (block.z - target.z)²)
    
    // Stores closest distance per player in GameState.scores[player_id]
}
```

**Scoring Logic:**
- Closest block to target wins the round
- Blocks out of bounds excluded from consideration
- Highest distance = lowest score (better)
- Tie possible if no blocks thrown

---

### 8. COMPONENT ARCHITECTURE

**ECS Components Used:**
```rust
// Location: /Users/sandro/Development/blocce/src/core/components.rs

Component Markers:
├── Block                  // All thrown blocks
├── TargetBlock           // The target to aim for
├── PlayerBlock { player } // Ownership tracking (0 or 1)
├── OutOfBounds           // Blocks off the map
├── ThrowingBlock         // Temporary throw indicator
├── ActiveBlock           // Currently tracked block
├── CameraTarget          // Block camera should follow
├── SettledBlock { time } // When block became settled
├── FadingOut { duration }// Fade animation tracking
├── FadingIn { duration } // Fade-in animation tracking
├── VictoryAnimation      // Supernova effect
├── MatchVictoryAnimation // Match winner effect
├── MainCamera            // The main game camera
└── GameText              // UI text elements
```

---

### 9. ANIMATION SYSTEMS

**Victory Animation (Supernova Effect):**
```rust
// Location: /Users/sandro/Development/blocce/src/gameplay/game.rs

Phase: VictoryAnimation (5 seconds)
1. Brighten winning block's emissive
2. Brighten target block too
3. Glow effect visible across arena
4. Fade out other blocks

Transition: VictoryAnimation → FadingOut (1.5 sec) → Playing
```

**Match Victory:**
```
Similar supernova for final winner of best-of-7
Display duration: 3 seconds
Press Space to restart
```

---

### 10. CAMERA & RENDERING

**Camera System:**
```rust
// Location: /Users/sandro/Development/blocce/src/rendering/camera.rs

Two Camera Modes:

1. Block Following (Active throw):
   - Offset from block: (0, 3, -5)
   - Follows with smooth lerp
   - Looks at thrown block

2. Orbital (Default):
   - Center point: (0, 0, 0)
   - Orbits based on throw_rotation
   - Distance: Configurable
   - Height: Configurable
   - Manual controls: Arrow keys (forward/back), W/S (height)

Post-Processing:
- Bloom intensity: Configurable
- Motion blur: Enabled
- Tonemapping: TonyMcMapface
```

**Lighting:**
```rust
- Directional light (sun/moon) with shadows
- Ambient light (subtle blue)
- Per-block point lights (player-colored, 300k intensity)
```

**File Location:**
- `/Users/sandro/Development/blocce/src/rendering/camera.rs`

---

### 11. CONFIGURATION SYSTEM

**Configuration File:** `config.yaml` (YAML format)

**Major Config Categories:**
```yaml
game:
  rounds_to_win: 4
  blocks_per_player: 3
  settle_duration: 1.0
  victory_display_duration: 5.0
  fade_in_duration: 0.4
  fade_out_duration: 1.5
  match_complete_delay: 3.0

physics:
  gravity: -9.81
  substeps: 12
  block:
    mass: 5.0
    friction: 0.7
    restitution: 0.5
    linear_damping: 0.2
    angular_damping: 0.5

throwing:
  power_multiplier: 20.0
  charge_rate: 0.8
  rotation_speed: 90.0
  default_angle: 45.0

arena:
  ground: { width: 30, height: 2, depth: 40 }
  target: { size: 0.3, spawn_zone: ±10 X, ±5 Z }
  boundaries: { x_max: 15, z_max: 20 }
  throwing_line: { x: 0, y: 2, z: -15 }

players:
  player1: { color: orange, emissive: bright_orange }
  player2: { color: blue, emissive: bright_blue }
```

**File Location:** `/Users/sandro/Development/blocce/src/core/config.rs`

---

### 12. MODULE STRUCTURE

```
src/
├── main.rs                 # App setup, state machines, system scheduling
├── lib.rs                  # Public exports
├── core/
│   ├── mod.rs             # Module re-exports
│   ├── components.rs      # ECS component markers
│   ├── resources.rs       # GameState, GamePhase
│   ├── states.rs          # AppState, PlayerType
│   ├── config.rs          # All configuration structures
│   ├── player.rs          # PlayerManager, PlayerConfig
│   └── fonts.rs           # Font resource management
├── physics/
│   ├── mod.rs
│   └── throwing.rs        # Velocity/spin calculations (unit tested)
├── gameplay/
│   ├── mod.rs
│   ├── game.rs            # Core game loop (900+ lines)
│   │   ├── Input handling
│   │   ├── Throwing physics
│   │   ├── Distance tracking
│   │   ├── Settlement detection
│   │   ├── Victory/match completion
│   │   └── Round transitions
│   └── arena.rs           # Ground, target, lighting setup
├── rendering/
│   ├── mod.rs
│   ├── camera.rs          # Camera and orbital systems
│   ├── light.rs           # Dynamic lighting for blocks
│   └── mod.rs
└── ui/
    ├── mod.rs
    ├── game_ui.rs         # Score display, controls text
    └── menu/
        ├── mod.rs
        ├── start_menu.rs
        ├── player_selection.rs
        ├── pause_menu.rs
        ├── countdown.rs
        ├── turn_notification.rs
        └── components.rs
```

---

### 13. SYSTEM SCHEDULING

**Update Loop (InGame state):**
```
Input Phase:
  → handle_input (throwing)
  → handle_pause_input

Physics Phase:
  → Bevy physics engine (12 substeps)

Game Logic Phase:
  → update_distances (real-time distance)
  → check_out_of_bounds
  → cleanup_out_of_bounds
  → check_throw_settled (settlement detection)
  → check_round_complete

Animation Phase:
  → start_victory_animation
  → animate_victory
  → transition_to_fadeout
  → fade_in_blocks
  → fade_out_blocks
  → start_next_round
  → respawn_target_for_round
  → start_match_victory_animation
  → animate_match_victory

Rendering Phase:
  → move_camera
  → attach_lights_to_blocks
  → update_block_lights

UI Phase:
  → update_ui
  → show_turn_notification
  → update_turn_notification
```

---

### 14. KEY INSIGHTS FOR CLIENT-SERVER ARCHITECTURE

**Simulation Requirements:**
1. **Deterministic Physics:** Yes - fixed substeps (12), enables lockstep or server authority
2. **Turn-Based:** Yes - natural fit for server-authoritative input processing
3. **Network Concerns:**
   - Block throwing must be server-authoritative (prevent speed hacking)
   - Settlement detection can be server-side (1-second threshold)
   - Distance calculation must be server-side (prevent winner hacking)
   - Round transitions fully server-controlled

**Recommended Network Model:**
- **Server:** Authoritative physics simulation, state management, victory determination
- **Client:** Input collection, prediction/interpolation, rendering only
- **Protocol:** UDP with client input → server sim → client state sync (20-30 Hz typical)

**Critical State to Sync:**
- Block positions/velocities after throws (verified by server)
- Game phase transitions
- Distance calculations at settlement time
- Round/match winners
- Player turn changes

---

### 15. TESTING INFRASTRUCTURE

**Physics Tests:** `/Users/sandro/Development/blocce/tests/physics_tests.rs`
- Block settling on ground
- Block-to-block stacking
- Wall collision (no penetration)

**Unit Tests in Code:**
- `/Users/sandro/Development/blocce/src/physics/throwing.rs`
  - Straight throw velocity
  - Angled throw velocity
  - Trajectory angle calculation
  - Unreachable target detection
  - Aim rotation calculation

---

## SUMMARY

BLOCCE is a **well-architected, physics-heavy, turn-based multiplayer game** built on Rust + Bevy + Avian3D. The codebase demonstrates:

- **Clean ECS design** with clear separation of concerns
- **Robust physics** with 12 substeps and comprehensive configuration
- **State machine** for game progression (Playing → Settling → Victory → Fade → Complete)
- **NO existing networking** - completely single-client, locally executed
- **Fully configurable** via YAML for rapid iteration
- **Good testing practices** with physics unit tests

For client-server architecture, the biggest advantage is the **deterministic physics** with fixed substeps, which enables either lockstep networking or server-authoritative simulation. The turn-based nature makes network coordination straightforward.

**Tool Call: Find `**/*.{js,ts,tsx,jsx,py,rs,go,c,cpp,h,java}`**
Status: Completed

/Users/sandro/Development/blocce/tests/physics_tests.rs
/Users/sandro/Development/blocce/src/physics/throwing.rs
/Users/sandro/Development/blocce/src/physics/mod.rs
/Users/sandro/Development/blocce/src/core/player.rs
/Users/sandro/Development/blocce/src/core/resources.rs
/Users/sandro/Development/blocce/src/gameplay/mod.rs
/Users/sandro/Development/blocce/src/lib.rs
/Users/sandro/Development/blocce/src/rendering/light.rs
/Users/sandro/Development/blocce/src/rendering/mod.rs
/Users/sandro/Development/blocce/src/rendering/camera.rs
/Users/sandro/Development/blocce/src/core/components.rs
/Users/sandro/Development/blocce/src/core/config.rs
/Users/sandro/Development/blocce/src/gameplay/arena.rs
/Users/sandro/Development/blocce/src/core/fonts.rs
/Users/sandro/Development/blocce/src/core/mod.rs
/Users/sandro/Development/blocce/src/gameplay/game.rs
/Users/sandro/Development/blocce/src/ui/game_ui.rs
/Users/sandro/Development/blocce/src/ui/menu/components.rs
/Users/sandro/Development/blocce/src/ui/menu/cleanup.rs
/Users/sandro/Development/blocce/src/main.rs
/Users/sandro/Development/blocce/src/core/states.rs
/Users/sandro/Development/blocce/src/ui/menu/mod.rs
/Users/sandro/Development/blocce/src/ui/menu/start_menu.rs
/Users/sandro/Development/blocce/src/ui/menu/player_selection.rs
/Users/sandro/Development/blocce/src/ui/menu/pause_menu.rs
/Users/sandro/Development/blocce/src/ui/menu/countdown.rs
/Users/sandro/Development/blocce/src/ui/menu/turn_notification.rs
/Users/sandro/Development/blocce/src/ui/mod.rs
/Users/sandro/Development/blocce/target/release/build/serde_core-cb096f6552ca4d8d/out/private.rs
/Users/sandro/Development/blocce/target/release/build/serde-05a99b9710c40b9a/out/private.rs
/Users/sandro/Development/blocce/target/release/build/thiserror-b435f0521ad05dcf/out/private.rs
/Users/sandro/Development/blocce/target/release/build/clang-sys-e8460848df756a27/out/macros.rs
/Users/sandro/Development/blocce/target/release/build/clang-sys-e8460848df756a27/out/common.rs
/Users/sandro/Development/blocce/target/release/build/clang-sys-e8460848df756a27/out/dynamic.rs
/Users/sandro/Development/blocce/target/release/build/coreaudio-sys-e807dcf478301c38/out/coreaudio.rs
/Users/sandro/Development/blocce/target/release/build/typenum-945584e583ca8d0f/out/tests.rs
/Users/sandro/Development/blocce/target/debug/build/serde-35ba4472f2ee923c/out/private.rs
/Users/sandro/Development/blocce/target/debug/build/serde_core-f365a46ceb6f4bcb/out/private.rs
/Users/sandro/Development/blocce/target/debug/build/thiserror-ea13c3218148d05b/out/private.rs
/Users/sandro/Development/blocce/target/debug/build/clang-sys-804920c977124c40/out/macros.rs
/Users/sandro/Development/blocce/target/debug/build/clang-sys-804920c977124c40/out/common.rs
/Users/sandro/Development/blocce/target/debug/build/clang-sys-804920c977124c40/out/dynamic.rs
/Users/sandro/Development/blocce/target/debug/build/typenum-06fb0dbd5dac965d/out/tests.rs
/Users/sandro/Development/blocce/target/debug/build/coreaudio-sys-bcb7ca1d9cd4829b/out/coreaudio.rs

**Tool Call: Find `**/*.md`**
Status: Completed

/Users/sandro/Development/blocce/README.md

**Tool Call: Find `**/package.json`**
Status: Completed

No files found

**Tool Call: grep "." /Users/sandro/Development/blocce/README.md**
Status: Completed

# BLOCCE
A competitive physics-based multiplayer game where two players take turns throwing colored blocks at a target. The player whose closest block lands nearest to the target wins the round!
## Overview
BLOCCE is a digital take on the classic game of bocce ball, built with Rust and the Bevy game engine. Two players compete in a best-of-7 match, throwing 3 blocks each per round toward a randomly positioned target. Realistic physics simulation with gravity, friction, and collisions makes every throw unique and challenging.
## Features
- **Physics-Based Gameplay**: Realistic 3D physics powered by Avian3D engine
- **Best-of-7 Match System**: First player to win 4 rounds wins the match
- **Dynamic Camera**: Smooth camera following that tracks your thrown blocks
- **Visual Effects**: Bloom post-processing, victory animations, and dynamic lighting
- **Fully Configurable**: Tune all game parameters through a YAML configuration file
- **Turn-Based Competition**: Strategic throwing with power control and aim rotation
## How to Play
### Game Rules
1. **Objective**: Get your blocks closest to the target
2. **Round Structure**:
   - Each player throws 3 blocks per round
   - Players alternate throws
   - After all blocks settle, the player with the closest block to the target wins the round
3. **Match Victory**: First player to win 4 rounds wins the match
4. **Out of Bounds**: Blocks that fall off the arena don't count for distance
### Controls
| Action | Key |
|--------|-----|
| Aim Left | `A` |
| Aim Right | `D` |
| Throw | `Space` (hold to charge power) |
| Camera Up/Down | `↑` / `↓` |
| Camera Raise/Lower | `W` / `S` |
| Restart Match | `Space` (after match ends) |
### Throwing Mechanics
- **Aim**: Use `A` and `D` to rotate your throw direction (±45° horizontal range)
- **Power**: Hold `Space` to charge your throw power (50% to 100% power)
  - Power increases over 1 second of holding
  - Release to throw
- **Physics**: Each block is thrown at a 45° vertical angle with random spin applied for realism
### Game Phases
1. **Playing**: Active throwing phase
2. **Settling**: Waiting for all blocks to come to rest
3. **Victory Animation**: Winner's closest block glows bright (supernova effect)
4. **Fade Out**: All blocks fade away before the next round
5. **Match Complete**: Final winner announced (press Space to restart)
## Installation & Running
### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (Edition 2021 or later)
### Build and Run
```bash
# Clone the repository
git clone <repository-url>
cd blocce
# Run in release mode (recommended for performance)
cargo run --release
# Or run in debug mode
cargo run
```
## Configuration
The game is highly configurable through the `config.yaml` file. You can customize:
### Game Rules
- Rounds to win (default: 4)
- Blocks per player (default: 3)
- Phase durations (settling wait, victory animation, fade out)
### Physics Parameters
- Gravity strength
- Block mass, friction, and bounciness
- Damping (air resistance)
- Settlement detection thresholds
### Throwing Mechanics
- Power multiplier
- Charge rate
- Rotation speed
- Throw angle
### Arena Layout
- Ground size
- Target spawn zone
- Boundary positions
### Visual Settings
- Player colors (Orange vs Blue)
- Bloom intensity and settings
- Camera position and behavior
- Lighting configuration
Example configuration snippet:
```yaml
game:
  rounds_to_win: 4
  blocks_per_player: 3
throwing:
  power_multiplier: 20.0
  charge_rate: 0.8
  rotation_speed: 45.0  # degrees per second
  default_angle: 45.0   # vertical throw angle
players:
  player1:
    color: { r: 1.0, g: 0.3, b: 0.0 }  # Orange
  player2:
    color: { r: 0.0, g: 0.4, b: 1.0 }  # Blue
```
See `config.yaml` for all available options and their defaults.
## Technical Details
### Built With
- **Language**: Rust (Edition 2021)
- **Game Engine**: [Bevy](https://bevyengine.org/) 0.17.2
- **Physics Engine**: [Avian3D](https://github.com/Jondolf/avian) 0.4
- **Architecture**: Entity Component System (ECS)
### Project Structure
```
blocce/
├── src/
│   ├── main.rs              # Entry point and app setup
│   ├── game.rs              # Core game loop and mechanics
│   ├── throwing.rs          # Physics calculations for throws
│   ├── arena.rs             # Arena and target spawning
│   ├── camera.rs            # Camera system
│   ├── ui.rs                # User interface
│   ├── components.rs        # ECS component definitions
│   ├── resources.rs         # Game state management
│   ├── config.rs            # Configuration loading
│   ├── player.rs            # Player configuration
│   └── light.rs             # Lighting system
├── config.yaml              # Game configuration
├── Cargo.toml               # Package manifest
└── README.md                # This file
```
### Key Systems
- **State Machine**: Clean game phase transitions (Playing → Settling → Victory → Fade → Complete)
- **Physics Simulation**: 12 substeps per frame for stable collision detection
- **Dynamic Lighting**: Each block carries its own point light with player-specific colors
- **Animation System**: Smooth brightness and transparency animations
- **Distance Tracking**: Real-time 3D distance calculation to determine winners
## Development
### Running Tests
```bash
cargo test
```
### Building for Release
```bash
cargo build --release
```
The optimized binary will be in `target/release/blocce`.
## License
See [LICENSE](LICENSE) and the license section of [README.md](README.md).
## Acknowledgments
- Built with [Bevy](https://bevyengine.org/) - A refreshingly simple data-driven game engine
- Physics by [Avian3D](https://github.com/Jondolf/avian) - ECS-driven physics engine for Bevy
---
Made with Rust 🦀

**Tool Call: Find `**/Cargo.toml`**
Status: Completed

/Users/sandro/Development/blocce/Cargo.toml

**Tool Call: grep "." /Users/sandro/Development/blocce/src/main.rs**
Status: Completed

use avian3d::dynamics::solver::SolverConfig;
use avian3d::prelude::*;
use bevy::prelude::*;
// Module declarations - organized by domain
mod core;
mod gameplay;
mod physics;
mod rendering;
mod ui;
// Imports from modules
use core::{AppState, GameConfiguration, GameFonts, GameSetup, GameState, PlayerManager};
use gameplay::{
    animate_match_victory, animate_victory, check_blocks_settled, check_out_of_bounds,
    check_round_complete, check_throw_settled, cleanup_out_of_bounds, detect_settled_camera_target,
    fade_in_blocks, fade_out_blocks, handle_input, handle_match_complete,
    remove_camera_target_after_settle, respawn_target_for_round, setup_arena, setup_lights,
    start_match_victory_animation, start_next_round, start_victory_animation,
    transition_to_fadeout, update_distances,
};
use rendering::{attach_lights_to_blocks, move_camera, setup_camera, update_block_lights};
use ui::{
    cleanup_menu_ui, handle_pause_input, handle_pause_menu, handle_player_selection,
    handle_start_menu, reset_game_state, setup_countdown, setup_pause_menu, setup_player_selection,
    setup_start_menu, setup_ui, show_turn_notification, update_button_colors, update_countdown,
    update_turn_notification, update_ui,
};
fn main() {
    // Load configuration from config.yaml
    let config = GameConfiguration::load_or_default();
    println!("=== BLOCCE - CONFIGURATION LOADED ===");
    println!(
        "Config: {} rounds to win, {} blocks per player",
        config.game.rounds_to_win, config.game.blocks_per_player
    );
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "blocce".to_string(),
            resolution: (1280, 720).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(PhysicsPlugins::default().with_length_unit(1.0));
    // Initialize resources from config
    let game_state = GameState::from_config(&config);
    let player_manager = PlayerManager::from_config(&config);
    let clear_color = ClearColor(config.rendering.clear_color.to_srgb());
    let gravity = Vec3::new(0.0, config.physics.gravity, 0.0);
    app.insert_resource(clear_color)
        .insert_resource(Gravity(gravity))
        .insert_resource(SubstepCount(config.physics.substeps))
        .insert_resource(SolverConfig {
            max_overlap_solve_speed: config.physics.max_overlap_solve_speed,
            contact_frequency_factor: 1.5,
            restitution_iterations: config.physics.restitution_iterations as usize,
            ..default()
        })
        .insert_resource(config)
        .insert_resource(game_state)
        .insert_resource(player_manager)
        .insert_resource(GameSetup::default())
        // Add state management
        .init_state::<AppState>()
        // Spawn arena and camera on startup for menu background
        .add_systems(
            Startup,
            (setup_camera, setup_arena, setup_lights, load_fonts),
        )
        // Loading systems - transition to StartMenu once fonts are loaded
        .add_systems(
            Update,
            check_fonts_loaded.run_if(in_state(AppState::Loading)),
        )
        // Start Menu systems
        .add_systems(OnEnter(AppState::StartMenu), setup_start_menu)
        .add_systems(OnExit(AppState::StartMenu), cleanup_menu_ui)
        .add_systems(
            Update,
            (handle_start_menu, update_button_colors).run_if(in_state(AppState::StartMenu)),
        )
        // Player Selection systems
        .add_systems(OnEnter(AppState::PlayerSelection), setup_player_selection)
        .add_systems(OnExit(AppState::PlayerSelection), cleanup_menu_ui)
        .add_systems(
            Update,
            (handle_player_selection, update_button_colors)
                .run_if(in_state(AppState::PlayerSelection)),
        )
        // Countdown systems
        .add_systems(OnEnter(AppState::Countdown), setup_countdown)
        .add_systems(OnExit(AppState::Countdown), cleanup_menu_ui)
        .add_systems(
            Update,
            update_countdown.run_if(in_state(AppState::Countdown)),
        )
        // Game initialization when entering InGame - just UI since arena/camera already exist
        .add_systems(OnEnter(AppState::InGame), setup_ui)
        // Game systems - only run during InGame state
        // Split into multiple groups due to tuple size limits
        .add_systems(
            Update,
            (
                handle_input,
                move_camera,
                update_distances,
                check_out_of_bounds,
                cleanup_out_of_bounds,
                check_throw_settled,
                check_round_complete,
                check_blocks_settled,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                start_victory_animation,
                animate_victory,
                transition_to_fadeout,
                fade_in_blocks,
                fade_out_blocks,
                start_next_round,
                respawn_target_for_round,
                start_match_victory_animation,
                animate_match_victory,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                handle_match_complete,
                detect_settled_camera_target,
                remove_camera_target_after_settle,
                attach_lights_to_blocks,
                update_block_lights,
                update_ui,
                show_turn_notification,
                update_turn_notification,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // Pause handling - runs during InGame state
        .add_systems(
            Update,
            handle_pause_input.run_if(in_state(AppState::InGame).or(in_state(AppState::PauseMenu))),
        )
        // Pause Menu systems
        .add_systems(OnEnter(AppState::PauseMenu), setup_pause_menu)
        .add_systems(OnExit(AppState::PauseMenu), cleanup_menu_ui)
        .add_systems(
            Update,
            (handle_pause_menu, update_button_colors).run_if(in_state(AppState::PauseMenu)),
        )
        // Cleanup when returning to menu
        .add_systems(
            OnEnter(AppState::StartMenu),
            (cleanup_game_entities, reset_game_state),
        )
        .run();
}
/// Load custom fonts at startup
fn load_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameFonts::load(&asset_server));
}
/// Check if fonts are loaded and transition to StartMenu
fn check_fonts_loaded(
    fonts: Option<Res<GameFonts>>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Some(fonts) = fonts {
        // Check if both fonts are loaded
        if asset_server.is_loaded_with_dependencies(&fonts.title)
            && asset_server.is_loaded_with_dependencies(&fonts.ui)
        {
            next_state.set(AppState::StartMenu);
        }
    }
}
/// Marker component for game entities that should be cleaned up
#[derive(Component)]
#[allow(dead_code)]
struct GameEntity;
/// Cleanup all game entities when returning to menu
/// Keep camera/arena/lights for background, only remove blocks and game UI
fn cleanup_game_entities(
    mut commands: Commands,
    blocks: Query<Entity, With<crate::core::Block>>,
    ui: Query<Entity, With<crate::core::GameText>>,
) {
    // Despawn blocks
    for entity in blocks.iter() {
        commands.entity(entity).despawn();
    }
    // Despawn game UI
    for entity in ui.iter() {
        commands.entity(entity).despawn();
    }
    // Keep camera, arena, and lights for menu background
}

**Tool Call: grep "." /Users/sandro/Development/blocce/src/core/states.rs**
Status: Completed

// App-level state management for menu navigation
use bevy::prelude::*;
/// Top-level application states for menu navigation
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    /// Loading state - ensures all resources are loaded before showing menu
    #[default]
    Loading,
    StartMenu,
    PlayerSelection,
    Countdown,
    InGame,
    PauseMenu,
}
/// Player type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum PlayerType {
    Human,
    CPU,
}
/// Game setup configuration from menu selections
#[derive(Resource, Debug, Clone)]
pub struct GameSetup {
    pub player1_type: PlayerType,
    pub player2_type: PlayerType,
}
impl Default for GameSetup {
    fn default() -> Self {
        Self {
            player1_type: PlayerType::Human,
            player2_type: PlayerType::Human,
        }
    }
}
/// Countdown timer resource
#[derive(Resource)]
pub struct CountdownTimer {
    pub remaining: f32,
}
impl CountdownTimer {
    pub fn new() -> Self {
        Self { remaining: 2.0 } // Changed to 2 seconds
    }
}
impl Default for CountdownTimer {
    fn default() -> Self {
        Self::new()
    }
}
/// Turn notification timer resource
#[derive(Resource)]
pub struct TurnNotification {
    pub timer: f32,
    pub showing: bool,
}
impl TurnNotification {
    pub fn new() -> Self {
        Self {
            timer: 1.5, // Show for 1.5 seconds
            showing: true,
        }
    }
}
impl Default for TurnNotification {
    fn default() -> Self {
        Self::new()
    }
}

**Tool Call: grep "." /Users/sandro/Development/blocce/src/core/components.rs**
Status: Completed

// Component markers for game entities
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

**Tool Call: grep "." /Users/sandro/Development/blocce/src/core/resources.rs**
Status: Completed

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

**Tool Call: grep "." /Users/sandro/Development/blocce/src/physics/mod.rs**
Status: Completed

// Physics calculations and simulation
pub mod throwing;
// Re-export public API
pub use throwing::*;

**Tool Call: grep "." /Users/sandro/Development/blocce/src/physics/throwing.rs**
Status: Completed

#![allow(dead_code)]
use bevy::prelude::*;
/// Parameters for a single throw
/// This structure separates throwing logic from entity spawning
#[derive(Debug, Clone)]
pub struct ThrowParameters {
    /// Vertical throw angle in degrees (pitch)
    pub angle: f32,
    /// Horizontal aim rotation in degrees (-45 to 45)
    pub rotation: f32,
    /// Throw power (0.0 to 1.0, typically 0.5 to 1.0)
    pub power: f32,
    /// Power multiplier (from config)
    pub power_multiplier: f32,
    /// Optional custom spin (if None, will use random spin)
    pub custom_spin: Option<Vec3>,
}
/// Result of throw calculation containing velocity vectors
#[derive(Debug, Clone)]
pub struct ThrowResult {
    /// Linear velocity to apply to the block
    pub linear_velocity: Vec3,
    /// Angular velocity to apply to the block
    pub angular_velocity: Vec3,
}
impl ThrowParameters {
    /// Create throw parameters from current game state
    pub fn new(angle: f32, rotation: f32, power: f32, power_multiplier: f32) -> Self {
        Self {
            angle,
            rotation,
            power,
            power_multiplier,
            custom_spin: None,
        }
    }
    /// Set custom spin for the throw
    pub fn with_spin(mut self, spin: Vec3) -> Self {
        self.custom_spin = Some(spin);
        self
    }
    /// Calculate the throw result (velocities)
    /// This is a pure function that can be tested independently
    pub fn calculate(&self, random_spin_ranges: Option<&Vec3>) -> ThrowResult {
        let linear_velocity =
            calculate_throw_velocity(self.angle, self.rotation, self.power, self.power_multiplier);
        let angular_velocity = if let Some(spin) = self.custom_spin {
            spin
        } else if let Some(ranges) = random_spin_ranges {
            calculate_random_spin(ranges)
        } else {
            Vec3::ZERO
        };
        ThrowResult {
            linear_velocity,
            angular_velocity,
        }
    }
}
/// Calculate linear velocity for a throw
///
/// # Arguments
/// * `angle` - Vertical throw angle in degrees (pitch)
/// * `rotation` - Horizontal aim rotation in degrees
/// * `power` - Throw power (0.0 to 1.0)
/// * `power_multiplier` - Multiplier to convert power to velocity magnitude
///
/// # Returns
/// Linear velocity vector in m/s
pub fn calculate_throw_velocity(
    angle: f32,
    rotation: f32,
    power: f32,
    power_multiplier: f32,
) -> Vec3 {
    let angle_rad = angle.to_radians();
    let rotation_rad = rotation.to_radians();
    let velocity_magnitude = power * power_multiplier;
    // Calculate velocity components based on angle and rotation
    // Angle: vertical (pitch) - how high the block is thrown
    // Rotation: horizontal (yaw) - left/right aim
    Vec3 {
        x: angle_rad.cos() * velocity_magnitude * rotation_rad.sin(),
        y: angle_rad.sin() * velocity_magnitude,
        z: angle_rad.cos() * velocity_magnitude * rotation_rad.cos(),
    }
}
/// Calculate random spin for a thrown block
///
/// # Arguments
/// * `ranges` - Vec3 containing max spin ranges for each axis in radians/sec
///
/// # Returns
/// Angular velocity vector in radians/second
pub fn calculate_random_spin(ranges: &Vec3) -> Vec3 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    Vec3 {
        x: (rng.gen::<f32>() - 0.5) * ranges.x,
        y: (rng.gen::<f32>() - 0.5) * ranges.y,
        z: (rng.gen::<f32>() - 0.5) * ranges.z,
    }
}
/// Calculate required angle to hit a target at a given distance
/// This is useful for AI opponents to aim at the target
/// Uses physics trajectory equations
///
/// # Arguments
/// * `distance` - Horizontal distance to target (m)
/// * `height_diff` - Vertical height difference (target_y - start_y)
/// * `velocity` - Initial velocity magnitude (m/s)
/// * `gravity` - Gravity magnitude (positive, typically 9.81)
///
/// # Returns
/// Optimal angle in degrees, or None if target is unreachable
pub fn calculate_trajectory_angle(
    distance: f32,
    height_diff: f32,
    velocity: f32,
    gravity: f32,
) -> Option<f32> {
    // Using trajectory equation: tan(θ) = (v² ± sqrt(v⁴ - g(gx² + 2yv²))) / gx
    // We use the lower angle solution (more direct shot)
    let v_squared = velocity * velocity;
    let v_fourth = v_squared * v_squared;
    let g_abs = gravity.abs();
    let discriminant =
        v_fourth - g_abs * (g_abs * distance * distance + 2.0 * height_diff * v_squared);
    if discriminant < 0.0 {
        // Target is unreachable with this velocity
        return None;
    }
    let numerator = v_squared - discriminant.sqrt();
    let denominator = g_abs * distance;
    if denominator == 0.0 {
        return Some(90.0); // Straight up if no horizontal distance
    }
    let angle_rad = (numerator / denominator).atan();
    Some(angle_rad.to_degrees())
}
/// Calculate horizontal rotation needed to aim at a target
///
/// # Arguments
/// * `start_pos` - Starting position of the throw
/// * `target_pos` - Target position to aim at
///
/// # Returns
/// Rotation angle in degrees
pub fn calculate_aim_rotation(start_pos: Vec3, target_pos: Vec3) -> f32 {
    let dx = target_pos.x - start_pos.x;
    let dz = target_pos.z - start_pos.z;
    // Calculate angle in radians, then convert to degrees
    let angle_rad = dx.atan2(dz);
    angle_rad.to_degrees()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_straight_throw() {
        // 45 degree angle, no rotation, half power
        let velocity = calculate_throw_velocity(45.0, 0.0, 0.5, 20.0);
        // At 45 degrees with rotation 0, x should be ~0
        assert!(velocity.x.abs() < 0.01);
        // Y and Z should be equal at 45 degrees
        assert!((velocity.y - velocity.z).abs() < 0.01);
        // Magnitude should be power * multiplier
        let magnitude = (velocity.x.powi(2) + velocity.y.powi(2) + velocity.z.powi(2)).sqrt();
        assert!((magnitude - 10.0).abs() < 0.1); // 0.5 * 20.0 = 10.0
    }
    #[test]
    fn test_angled_throw() {
        // 45 degree angle, 45 degree right rotation
        let velocity = calculate_throw_velocity(45.0, 45.0, 1.0, 20.0);
        // X should be positive (right)
        assert!(velocity.x > 0.0);
        // Z should be positive (forward)
        assert!(velocity.z > 0.0);
        // Y should be positive (up)
        assert!(velocity.y > 0.0);
    }
    #[test]
    fn test_throw_parameters() {
        let params = ThrowParameters::new(45.0, 0.0, 1.0, 20.0);
        let ranges = Vec3::new(5.0, 3.0, 5.0);
        let result = params.calculate(Some(&ranges));
        // Velocity should be non-zero
        assert!(result.linear_velocity.length() > 0.0);
        // Angular velocity should be set (random spin)
        assert!(result.angular_velocity.length() > 0.0);
    }
    #[test]
    fn test_custom_spin() {
        let custom_spin = Vec3::new(1.0, 2.0, 3.0);
        let params = ThrowParameters::new(45.0, 0.0, 1.0, 20.0).with_spin(custom_spin);
        let result = params.calculate(None);
        // Should use custom spin, not random
        assert_eq!(result.angular_velocity, custom_spin);
    }
    #[test]
    fn test_trajectory_angle_calculation() {
        // Test with reasonable game values
        let distance = 23.0; // ~distance from throwing line to target
        let height_diff = 0.0; // Target at same height
        let velocity = 20.0; // Max velocity
        let gravity = 9.81;
        let angle = calculate_trajectory_angle(distance, height_diff, velocity, gravity);
        assert!(angle.is_some());
        let angle = angle.unwrap();
        assert!(angle > 0.0 && angle < 90.0);
    }
    #[test]
    fn test_unreachable_target() {
        // Target too far away
        let angle = calculate_trajectory_angle(100.0, 0.0, 10.0, 9.81);
        assert!(angle.is_none());
    }
    #[test]
    fn test_aim_rotation() {
        // Test aiming straight forward
        let rotation = calculate_aim_rotation(Vec3::new(0.0, 0.0, -15.0), Vec3::new(0.0, 0.0, 8.0));
        assert!((rotation - 0.0).abs() < 0.1);
        // Test aiming to the right
        let rotation =
            calculate_aim_rotation(Vec3::new(0.0, 0.0, -15.0), Vec3::new(10.0, 0.0, 8.0));
        assert!(rotation > 0.0);
        // Test aiming to the left
        let rotation =
            calculate_aim_rotation(Vec3::new(0.0, 0.0, -15.0), Vec3::new(-10.0, 0.0, 8.0));
        assert!(rotation < 0.0);
    }
}

**Tool Call: grep | head -200 "." /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Completed

// Complete game logic - state management, progression, animations
use avian3d::prelude::*;
use bevy::prelude::*;
use super::arena::spawn_target;
use crate::core::{
    ActiveBlock, Block, CameraTarget, FadingIn, FadingOut, GameConfiguration, GamePhase, GameState,
    MatchVictoryAnimation, OutOfBounds, PlayerBlock, PlayerManager, SettledBlock, TargetBlock,
    ThrowingBlock, VictoryAnimation,
};
use crate::physics::ThrowParameters;
// =============================================================================
// INPUT & THROWING
// =============================================================================
/// Handle player input for aiming and throwing
#[allow(clippy::too_many_arguments)]
pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
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
    if game_state.phase != GamePhase::Playing {
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
fn throw_block(
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
    let mut entity = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(block_size, block_size, block_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: player_config.color,
            emissive: player_config.emissive,
            ..default()
        })),
        Transform::from_xyz(start_pos.x, start_pos.y, start_pos.z),
        RigidBody::Dynamic,
        Collider::cuboid(block_size, block_size, block_size),
        Mass(physics_cfg.mass),
        LinearVelocity(throw_result.linear_velocity),
        AngularVelocity(throw_result.angular_velocity),
        Friction::new(physics_cfg.friction),
        Restitution::new(physics_cfg.restitution),
        Block,
        PlayerBlock {
            player: game_state.current_player,
        },
    ));
    entity.insert((
        LinearDamping(physics_cfg.linear_damping),
        AngularDamping(physics_cfg.angular_damping),
        GravityScale(physics_cfg.gravity_scale),
        SweptCcd::default(),
        SleepingDisabled,
        ActiveBlock,
        CameraTarget, // Mark this block for camera following
        FadingIn {
            start_time: time.elapsed_secs(),
            duration: config.game.fade_in_duration,
            start_alpha: 0.0,
        },
    ));
}
// =============================================================================
// DISTANCE TRACKING
// =============================================================================
/// Continuously track all blocks' distances to the target
#[allow(clippy::type_complexity)]
pub fn update_distances(
    mut game_state: ResMut<GameState>,
    player_blocks: Query<(&Transform, &PlayerBlock), (With<Block>, Without<OutOfBounds>)>,
    target_query: Query<&Transform, With<TargetBlock>>,
) {
    if let Ok(target_transform) = target_query.single() {
        let target_pos = target_transform.translation;
        // Track closest distance for each player
        let mut closest_distances = [f32::MAX, f32::MAX];
        for (transform, player_block) in player_blocks.iter() {
            let distance = transform.translation.distance(target_pos);
            let player_id = player_block.player;
            if distance < closest_distances[player_id] {
                closest_distances[player_id] = distance;
            }
        }
        // Update scores
        for (player_id, &closest_distance) in closest_distances.iter().enumerate() {
            if closest_distance < f32::MAX {
                game_state.scores[player_id] = closest_distance;
            } else if game_state.blocks_thrown[player_id] > 0 {
                game_state.scores[player_id] = f32::MAX; // All blocks out of bounds
            } else {
                game_state.scores[player_id] = 0.0; // No blocks thrown yet
            }
        }
    }
}
// =============================================================================
// OUT OF BOUNDS HANDLING
// =============================================================================
/// Check if blocks have fallen off the map
#[allow(clippy::type_complexity)]
pub fn check_out_of_bounds(
    mut commands: Commands,
    blocks: Query<(Entity, &Transform, &PlayerBlock), (With<Block>, Without<OutOfBounds>)>,
    config: Res<GameConfiguration>,
) {
    let boundaries = &config.arena.boundaries;
    for (entity, transform, _player_block) in blocks.iter() {
        let pos = transform.translation;
        // Boundaries from config

**Tool Call: grep | head -200 "." /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Completed

// Complete game logic - state management, progression, animations
use avian3d::prelude::*;
use bevy::prelude::*;
use super::arena::spawn_target;
use crate::core::{
    ActiveBlock, Block, CameraTarget, FadingIn, FadingOut, GameConfiguration, GamePhase, GameState,
    MatchVictoryAnimation, OutOfBounds, PlayerBlock, PlayerManager, SettledBlock, TargetBlock,
    ThrowingBlock, VictoryAnimation,
};
use crate::physics::ThrowParameters;
// =============================================================================
// INPUT & THROWING
// =============================================================================
/// Handle player input for aiming and throwing
#[allow(clippy::too_many_arguments)]
pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
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
    if game_state.phase != GamePhase::Playing {
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
fn throw_block(
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
    let mut entity = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(block_size, block_size, block_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: player_config.color,
            emissive: player_config.emissive,
            ..default()
        })),
        Transform::from_xyz(start_pos.x, start_pos.y, start_pos.z),
        RigidBody::Dynamic,
        Collider::cuboid(block_size, block_size, block_size),
        Mass(physics_cfg.mass),
        LinearVelocity(throw_result.linear_velocity),
        AngularVelocity(throw_result.angular_velocity),
        Friction::new(physics_cfg.friction),
        Restitution::new(physics_cfg.restitution),
        Block,
        PlayerBlock {
            player: game_state.current_player,
        },
    ));
    entity.insert((
        LinearDamping(physics_cfg.linear_damping),
        AngularDamping(physics_cfg.angular_damping),
        GravityScale(physics_cfg.gravity_scale),
        SweptCcd::default(),
        SleepingDisabled,
        ActiveBlock,
        CameraTarget, // Mark this block for camera following
        FadingIn {
            start_time: time.elapsed_secs(),
            duration: config.game.fade_in_duration,
            start_alpha: 0.0,
        },
    ));
}
// =============================================================================
// DISTANCE TRACKING
// =============================================================================
/// Continuously track all blocks' distances to the target
#[allow(clippy::type_complexity)]
pub fn update_distances(
    mut game_state: ResMut<GameState>,
    player_blocks: Query<(&Transform, &PlayerBlock), (With<Block>, Without<OutOfBounds>)>,
    target_query: Query<&Transform, With<TargetBlock>>,
) {
    if let Ok(target_transform) = target_query.single() {
        let target_pos = target_transform.translation;
        // Track closest distance for each player
        let mut closest_distances = [f32::MAX, f32::MAX];
        for (transform, player_block) in player_blocks.iter() {
            let distance = transform.translation.distance(target_pos);
            let player_id = player_block.player;
            if distance < closest_distances[player_id] {
                closest_distances[player_id] = distance;
            }
        }
        // Update scores
        for (player_id, &closest_distance) in closest_distances.iter().enumerate() {
            if closest_distance < f32::MAX {
                game_state.scores[player_id] = closest_distance;
            } else if game_state.blocks_thrown[player_id] > 0 {
                game_state.scores[player_id] = f32::MAX; // All blocks out of bounds
            } else {
                game_state.scores[player_id] = 0.0; // No blocks thrown yet
            }
        }
    }
}
// =============================================================================
// OUT OF BOUNDS HANDLING
// =============================================================================
/// Check if blocks have fallen off the map
#[allow(clippy::type_complexity)]
pub fn check_out_of_bounds(
    mut commands: Commands,
    blocks: Query<(Entity, &Transform, &PlayerBlock), (With<Block>, Without<OutOfBounds>)>,
    config: Res<GameConfiguration>,
) {
    let boundaries = &config.arena.boundaries;
    for (entity, transform, _player_block) in blocks.iter() {
        let pos = transform.translation;
        // Boundaries from config

**Tool Call: grep | head -300 "check_out_of_bounds|check_throw_settled|check_round_complete|check_blocks_settled" /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Completed

pub fn check_out_of_bounds(
pub fn check_throw_settled(
pub fn check_round_complete(
    // This is now handled by check_throw_settled, but keep as fallback
pub fn check_blocks_settled(

**Tool Call: grep | head -150 "pub fn check_throw_settled" /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Failed

```
<tool_use_error>InputValidationError: Grep failed due to the following issue:
An unexpected parameter `A` was provided</tool_use_error>
```

**Tool Call: grep -A 100 | head -150 "pub fn check_throw_settled" /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Completed

pub fn check_throw_settled(
    mut game_state: ResMut<GameState>,
    config: Res<GameConfiguration>,
    blocks: Query<(&LinearVelocity, &AngularVelocity), With<Block>>,
    out_of_bounds: Query<Entity, (With<Block>, With<OutOfBounds>)>,
    time: Res<Time>,
) {
    // Only check during Playing phase when blocks are not settled
    if game_state.phase != GamePhase::Playing || game_state.all_blocks_settled {
        return;
    }
    // Check if any blocks exist and if they're all settled
    if blocks.is_empty() {
        game_state.all_blocks_settled = true;
        game_state.settling_start_time = None;
        return;
    }
    let settlement = &config.physics.settlement;
    let mut all_settled = true;
    for (lin_vel, ang_vel) in blocks.iter() {
        let speed = lin_vel.length();
        let ang_speed = ang_vel.length();
        if speed > settlement.velocity_threshold
            || ang_speed > settlement.angular_velocity_threshold
        {
            all_settled = false;
            break;
        }
    }
    // Wait for OOB blocks to despawn
    if out_of_bounds.iter().count() > 0 {
        all_settled = false;
    }
    if !all_settled {
        game_state.settling_start_time = None;
        return;
    }
    // Track settling time
    let current_time = time.elapsed_secs();
    if game_state.settling_start_time.is_none() {
        game_state.settling_start_time = Some(current_time);
        return;
    }
    let time_settled = current_time - game_state.settling_start_time.unwrap();
    if time_settled < config.game.settle_duration {
        return;
    }
    // Blocks settled after this throw!
    game_state.all_blocks_settled = true;
    game_state.settling_start_time = None;
    println!("Blocks settled. Ready for next throw.");
    let blocks_per_player = config.game.blocks_per_player;
    // Check if current player finished their turns
    if game_state.blocks_thrown[game_state.current_player] >= blocks_per_player {
        // Check if other player also finished
        let other_player = 1 - game_state.current_player;
        if game_state.blocks_thrown[other_player] >= blocks_per_player {
            // Both players done - move to BlocksSettling phase for final scoring
            println!("\n=== ALL BLOCKS THROWN ===");
            println!("Waiting for final settlement...");
            game_state.transition_to_phase(GamePhase::BlocksSettling, time.elapsed_secs());
        } else {
            // Switch to other player
            game_state.current_player = other_player;
            game_state.throw_rotation = config.throwing.default_rotation;
            game_state.throw_power = config.throwing.min_power;
            println!(
                "\n=== SWITCHING TO PLAYER {} ===",
                game_state.current_player + 1
            );
        }
    }
}
/// Check if round is complete (both players threw all blocks each)
pub fn check_round_complete(
    mut game_state: ResMut<GameState>,
    config: Res<GameConfiguration>,
    time: Res<Time>,
) {
    if game_state.phase != GamePhase::Playing {
        return;
    }
    let blocks_per_player = config.game.blocks_per_player;
    // This is now handled by check_throw_settled, but keep as fallback
    if game_state.blocks_thrown[0] >= blocks_per_player
        && game_state.blocks_thrown[1] >= blocks_per_player
        && game_state.all_blocks_settled

**Tool Call: grep | head -150 "." /Users/sandro/Development/blocce/src/gameplay/arena.rs**
Status: Completed

// Arena and course setup
use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;
use crate::core::{FadingIn, GameConfiguration, TargetBlock};
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
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(ground.width, ground.height, ground.depth))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.15, 0.15),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(0.0, ground.y_position, 0.0),
        RigidBody::Static,
        Collider::cuboid(ground.width, ground.height, ground.depth),
        Friction::new(physics.friction),
        Restitution::new(physics.restitution),
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
    // Invisible floor to catch falling blocks
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(safety.width, safety.height, safety.depth))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.0, 0.0, 0.0), // Fully transparent
            alpha_mode: bevy::prelude::AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, safety.y_position, 0.0),
        RigidBody::Static,
        Collider::cuboid(safety.width, safety.height, safety.depth),
        Restitution::new(0.0), // No bounce
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
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(
                target_cfg.size,
                target_cfg.size,
                target_cfg.size,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),
                emissive: emissive_cfg.to_linear(),
                ..default()
            })),
            Transform::from_xyz(target_x, target_cfg.y_position, target_z),
            TargetBlock,
            RigidBody::Static,
            Collider::cuboid(target_cfg.size, target_cfg.size, target_cfg.size),
            FadingIn {
                start_time: time.elapsed_secs(),
                duration: config.game.fade_in_duration,
                start_alpha: 0.0,
            },
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

**Tool Call: grep | head -200 "." /Users/sandro/Development/blocce/src/core/config.rs**
Status: Completed

use bevy::prelude::*;
use serde::Deserialize;
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
    pub fn load() -> Result<Self, String> {
        let config_path = "config.yaml";
        let config_str = fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read {}: {}", config_path, e))?;
        serde_yaml::from_str(&config_str)
            .map_err(|e| format!("Failed to parse {}: {}", config_path, e))
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

**Tool Call: grep "." /Users/sandro/Development/blocce/src/core/player.rs**
Status: Completed

// Player configuration and management
use super::config::{GameConfiguration, PlayerConfigData};
use bevy::prelude::*;
/// Configuration for a player including visual appearance
#[derive(Clone, Debug)]
pub struct PlayerConfig {
    pub name: String,
    pub color: Color,         // Base color for blocks and UI
    pub emissive: LinearRgba, // Emissive glow for blocks
}
impl PlayerConfig {
    /// Create from config data
    pub fn from_config_data(data: &PlayerConfigData) -> Self {
        Self {
            name: data.name.clone(),
            color: data.color.to_srgb(),
            emissive: data.emissive.to_linear(),
        }
    }
}
/// Resource to manage players in the game
#[derive(Resource)]
pub struct PlayerManager {
    pub players: Vec<PlayerConfig>,
}
impl PlayerManager {
    /// Create from game configuration
    pub fn from_config(config: &GameConfiguration) -> Self {
        Self {
            players: config
                .players
                .iter()
                .map(PlayerConfig::from_config_data)
                .collect(),
        }
    }
    /// Get configuration for a specific player
    pub fn get_player(&self, id: usize) -> Option<&PlayerConfig> {
        self.players.get(id)
    }
}

**Tool Call: grep --type=rs "network|server|client|multiplayer|websocket|socket|sync|rpc"**
Status: Completed

No files found

**Tool Call: Find `**/Cargo.toml`**
Status: Completed

/Users/sandro/Development/blocce/Cargo.toml

**Tool Call: grep "." /Users/sandro/Development/blocce/Cargo.toml**
Status: Completed

[package]
name = "blocce"
version = "0.1.0"
edition = "2021"
[dependencies]
bevy = "0.17.2"
rand = "0.8"
avian3d = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
[profile.release]
opt-level = "s"

**Tool Call: grep | head -20 --type=rs "Collider|collision|RigidBody|Physics|Velocity"**
Status: Completed

No files found

**Tool Call: Find `/Users/sandro/Development/blocce/src/gameplay/game.rs`**
Status: Completed

/Users/sandro/Development/blocce/src/gameplay/game.rs

**Tool Call: grep | head -300 -P "pub fn (start_victory|animate_victory|check_blocks_settled|start_match_victory|animate_match_victory|transition_to_fadeout|fade_out_blocks|start_next_round)" /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Completed

pub fn check_blocks_settled(
pub fn start_victory_animation(
pub fn animate_victory(
pub fn transition_to_fadeout(
pub fn fade_out_blocks(
pub fn start_next_round(
pub fn start_match_victory_animation(
pub fn animate_match_victory(

**Tool Call: grep | head -200 -P "pub fn start_victory_animation" /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Failed

```
<tool_use_error>InputValidationError: Grep failed due to the following issue:
An unexpected parameter `B` was provided</tool_use_error>
```

**Tool Call: grep -B 2 | head -200 -P "pub fn start_victory_animation" /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Completed

/// Start victory animation when entering phase
#[allow(clippy::type_complexity)]
pub fn start_victory_animation(

**Tool Call: grep "^pub fn" /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Completed

pub fn handle_input(
pub fn update_distances(
pub fn check_out_of_bounds(
pub fn cleanup_out_of_bounds(
pub fn check_throw_settled(
pub fn check_round_complete(
pub fn check_blocks_settled(
pub fn start_victory_animation(
pub fn animate_victory(
pub fn transition_to_fadeout(
pub fn fade_in_blocks(
pub fn fade_out_blocks(
pub fn detect_settled_camera_target(
pub fn remove_camera_target_after_settle(
pub fn start_next_round(
pub fn respawn_target_for_round(
pub fn despawn_target_for_round_end(
pub fn start_match_victory_animation(
pub fn animate_match_victory(
pub fn handle_match_complete(

**Tool Call: grep | head -400 "VICTORY|VictoryAnimation|match_complete|start_match_victory" /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Completed

MatchVictoryAnimation, OutOfBounds, PlayerBlock, PlayerManager, SettledBlock, TargetBlock,
    ThrowingBlock, VictoryAnimation,
        game_state.transition_to_phase(GamePhase::VictoryAnimation, time.elapsed_secs());
// VICTORY ANIMATION (SUPERNOVA)
    player_blocks: Query<(Entity, &Transform), (With<Block>, Without<VictoryAnimation>)>,
    if game_state.phase != GamePhase::VictoryAnimation {
            commands.entity(entity).insert(VictoryAnimation {
        commands.entity(target_entity).insert(VictoryAnimation {
        &VictoryAnimation,
        (&VictoryAnimation, &MeshMaterial3d<StandardMaterial>),
    animating: Query<(Entity, &MeshMaterial3d<StandardMaterial>), With<VictoryAnimation>>,
    if game_state.phase != GamePhase::VictoryAnimation {
            commands.entity(entity).remove::<VictoryAnimation>();
    if game_state.phase == GamePhase::VictoryAnimation {
pub fn start_match_victory_animation(
    player_blocks: Query<(Entity, &PlayerBlock), (With<Block>, Without<MatchVictoryAnimation>)>,
            .insert(MatchVictoryAnimation { winner_player });
        &MatchVictoryAnimation,
pub fn handle_match_complete(
    if time_in_phase > config.game.match_complete_delay && keyboard.just_pressed(KeyCode::Space) {

**Tool Call: grep | head -150 "." /Users/sandro/Development/blocce/src/rendering/camera.rs**
Status: Completed

// Camera setup and movement
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::{Bloom, BloomCompositeMode};
use bevy::post_process::motion_blur::MotionBlur;
use bevy::prelude::*;
use crate::core::{CameraTarget, GameConfiguration, GameState, MainCamera};
/// Setup the main camera with bloom and motion blur effects
pub fn setup_camera(mut commands: Commands, config: Res<GameConfiguration>) {
    let initial = &config.camera.initial;
    let bloom_cfg = &config.rendering.bloom;
    let motion_blur_cfg = &config.rendering.motion_blur;
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(initial.x, initial.y, initial.z).looking_at(
            Vec3::new(initial.look_at_x, initial.look_at_y, initial.look_at_z),
            Vec3::Y,
        ),
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: bloom_cfg.intensity,
            low_frequency_boost: bloom_cfg.low_frequency_boost,
            low_frequency_boost_curvature: bloom_cfg.low_frequency_boost_curvature,
            high_pass_frequency: bloom_cfg.high_pass_frequency,
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        },
        MotionBlur {
            shutter_angle: motion_blur_cfg.shutter_angle,
            samples: motion_blur_cfg.samples,
        },
        MainCamera,
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

**Tool Call: grep | head -400 "." /Users/sandro/Development/blocce/src/core/config.rs**
Status: Completed

use bevy::prelude::*;
use serde::Deserialize;
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
    pub fn load() -> Result<Self, String> {
        let config_path = "config.yaml";
        let config_str = fs::read_to_string(config_path)
            .map_err(|e| format!("Failed to read {}: {}", config_path, e))?;
        serde_yaml::from_str(&config_str)
            .map_err(|e| format!("Failed to parse {}: {}", config_path, e))
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

**Tool Call: grep "." /Users/sandro/Development/blocce/tests/physics_tests.rs**
Status: Completed

use avian3d::dynamics::solver::SolverConfig;
use avian3d::prelude::*;
use bevy::prelude::*;
// Import the game's collision layer enum
// Note: You may need to expose this in lib.rs or make it public
// For now, we'll define it locally
#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
enum GameLayer {
    #[default]
    Ground,
    Block,
    Target,
}
#[test]
fn test_block_collision_with_ground() {
    let mut app = App::new();
    // Add plugins needed for physics testing
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
        bevy::render::RenderPlugin::default(),
        bevy::render::mesh::MeshPlugin,
    ))
    .add_plugins(PhysicsPlugins::default().with_length_unit(1.0))
    .insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)))
    .insert_resource(SubstepCount(12))
    .insert_resource(SolverConfig {
        max_overlap_solve_speed: 5.0,
        contact_frequency_factor: 1.5,
        restitution_iterations: 4,
        ..default()
    });
    // Spawn ground
    let ground_height = 2.0;
    let ground_width = 30.0;
    let ground_depth = 40.0;
    app.world_mut().spawn((
        Transform::from_xyz(0.0, -ground_height / 2.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(ground_width / 2.0, ground_height / 2.0, ground_depth / 2.0),
        CollisionLayers::new(GameLayer::Ground, [GameLayer::Block, GameLayer::Target]),
        Friction::new(0.8),
        Restitution::new(0.2),
    ));
    // Spawn a test block above the ground
    let block_size = 1.0;
    let start_y = 5.0; // Start 5 units above ground
    let block_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, start_y, 0.0),
            RigidBody::Dynamic,
            Collider::cuboid(block_size / 2.0, block_size / 2.0, block_size / 2.0),
            CollisionLayers::new(GameLayer::Block, [GameLayer::Ground, GameLayer::Block]),
            Mass(5.0),
            LinearVelocity(Vec3::ZERO),
            AngularVelocity(Vec3::ZERO),
            Friction::new(0.7),
            Restitution::new(0.3),
            LinearDamping(0.2),
            AngularDamping(0.5),
            GravityScale(1.0),
            SweptCcd::default(),
            SleepingDisabled,
        ))
        .id();
    // Run physics simulation for 5 seconds (at 60fps = 300 frames)
    for _ in 0..300 {
        app.update();
    }
    // Check that block has settled on the ground
    let transform = app.world().entity(block_entity).get::<Transform>().unwrap();
    let position = transform.translation;
    println!("Test: Block final position: {:?}", position);
    // Expected: block center should be at y = block_size/2 = 0.5
    // Allow for small tolerance due to physics simulation
    let expected_y = block_size / 2.0;
    let tolerance = 0.1;
    assert!(
        (position.y - expected_y).abs() < tolerance,
        "Block should rest at y={} (within {} tolerance), but is at y={}. Block is sinking!",
        expected_y,
        tolerance,
        position.y
    );
    // Check that block has stopped moving (settled)
    let lin_vel = app
        .world()
        .entity(block_entity)
        .get::<LinearVelocity>()
        .unwrap();
    assert!(
        lin_vel.length() < 0.1,
        "Block should have settled (velocity < 0.1), but velocity is {}",
        lin_vel.length()
    );
}
#[test]
fn test_block_to_block_collision() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(PhysicsPlugins::default().with_length_unit(1.0))
        .insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)))
        .insert_resource(SubstepCount(12))
        .insert_resource(SolverConfig {
            max_overlap_solve_speed: 5.0,
            contact_frequency_factor: 1.5,
            restitution_iterations: 4,
            ..default()
        });
    // Spawn ground
    let ground_height = 2.0;
    app.world_mut().spawn((
        Transform::from_xyz(0.0, -ground_height / 2.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(15.0, ground_height / 2.0, 20.0),
        CollisionLayers::new(GameLayer::Ground, [GameLayer::Block, GameLayer::Target]),
        Friction::new(0.8),
        Restitution::new(0.2),
    ));
    // Spawn first block on the ground
    let block_size = 1.0;
    let block1_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, block_size / 2.0, 0.0), // Resting on ground
            RigidBody::Dynamic,
            Collider::cuboid(block_size / 2.0, block_size / 2.0, block_size / 2.0),
            CollisionLayers::new(GameLayer::Block, [GameLayer::Ground, GameLayer::Block]),
            Mass(5.0),
            LinearVelocity(Vec3::ZERO),
            AngularVelocity(Vec3::ZERO),
            Friction::new(0.7),
            Restitution::new(0.3),
            GravityScale(1.0),
            SweptCcd::default(),
            SleepingDisabled,
        ))
        .id();
    // Spawn second block above the first
    let block2_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, block_size / 2.0 + block_size + 2.0, 0.0), // 2 units above first block
            RigidBody::Dynamic,
            Collider::cuboid(block_size / 2.0, block_size / 2.0, block_size / 2.0),
            CollisionLayers::new(GameLayer::Block, [GameLayer::Ground, GameLayer::Block]),
            Mass(5.0),
            LinearVelocity(Vec3::ZERO),
            AngularVelocity(Vec3::ZERO),
            Friction::new(0.7),
            Restitution::new(0.3),
            GravityScale(1.0),
            SweptCcd::default(),
            SleepingDisabled,
        ))
        .id();
    // Run simulation
    for _ in 0..300 {
        app.update();
    }
    // Check positions
    let block1_pos = app
        .world()
        .entity(block1_entity)
        .get::<Transform>()
        .unwrap()
        .translation;
    let block2_pos = app
        .world()
        .entity(block2_entity)
        .get::<Transform>()
        .unwrap()
        .translation;
    println!("Test: Block 1 position: {:?}", block1_pos);
    println!("Test: Block 2 position: {:?}", block2_pos);
    // Block 2 should be stacked on top of block 1
    // Block 1 should be at y = 0.5 (on ground)
    // Block 2 should be at y = 1.5 (on top of block 1)
    let tolerance = 0.15;
    assert!(
        (block1_pos.y - 0.5).abs() < tolerance,
        "Block 1 should be on ground at y=0.5, but is at y={}",
        block1_pos.y
    );
    assert!(
        (block2_pos.y - 1.5).abs() < tolerance,
        "Block 2 should be stacked at y=1.5, but is at y={}. Blocks are not colliding!",
        block2_pos.y
    );
    // Blocks should be at same x, z position
    assert!(
        (block2_pos.x - block1_pos.x).abs() < 0.2,
        "Blocks should be aligned in X"
    );
}
#[test]
fn test_wall_collision() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(PhysicsPlugins::default().with_length_unit(1.0))
        .insert_resource(Gravity(Vec3::new(0.0, -9.81, 0.0)))
        .insert_resource(SubstepCount(12))
        .insert_resource(SolverConfig {
            max_overlap_solve_speed: 5.0,
            contact_frequency_factor: 1.5,
            restitution_iterations: 4,
            ..default()
        });
    // Spawn ground
    app.world_mut().spawn((
        Transform::from_xyz(0.0, -1.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(15.0, 1.0, 20.0),
        CollisionLayers::new(GameLayer::Ground, GameLayer::Block),
    ));
    // Spawn wall at x=10
    let wall_x = 10.0;
    app.world_mut().spawn((
        Transform::from_xyz(wall_x, 2.5, 0.0),
        RigidBody::Static,
        Collider::cuboid(1.0, 2.5, 20.0),
        CollisionLayers::new(GameLayer::Ground, GameLayer::Block),
        Friction::new(0.3),
        Restitution::new(0.5),
    ));
    // Spawn block moving towards the wall
    let block_size = 1.0;
    let block_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(5.0, block_size / 2.0, 0.0),
            RigidBody::Dynamic,
            Collider::cuboid(block_size / 2.0, block_size / 2.0, block_size / 2.0),
            CollisionLayers::new(GameLayer::Block, [GameLayer::Ground, GameLayer::Block]),
            Mass(5.0),
            LinearVelocity(Vec3::new(10.0, 0.0, 0.0)), // Moving towards wall at high speed
            AngularVelocity(Vec3::ZERO),
            Friction::new(0.7),
            Restitution::new(0.3),
            GravityScale(1.0),
            SweptCcd::default(),
            SleepingDisabled,
        ))
        .id();
    // Run simulation
    for _ in 0..300 {
        app.update();
    }
    // Check that block didn't pass through wall
    let block_pos = app
        .world()
        .entity(block_entity)
        .get::<Transform>()
        .unwrap()
        .translation;
    println!(
        "Test: Block final position after wall collision: {:?}",
        block_pos
    );
    // Block should not have passed through the wall (wall is at x=10, block is 1 unit wide)
    // So block center should be less than wall_x - block_size/2 - wall_thickness/2
    let max_x = wall_x - block_size / 2.0 - 1.0; // wall thickness is 2, so half is 1
    assert!(
        block_pos.x < max_x,
        "Block passed through wall! Position x={}, should be less than {}",
        block_pos.x,
        max_x
    );
}

**Tool Call: grep "." /Users/sandro/Development/blocce/src/lib.rs**
Status: Completed

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

**Tool Call: grep | head -600 "pub fn (start_victory|animate_victory|check_blocks|transition_to|fade|start_next|respawn_target|start_match_victory|animate_match_victory|handle_match_complete)" /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Completed

pub fn check_blocks_settled(
pub fn start_victory_animation(
pub fn animate_victory(
pub fn transition_to_fadeout(
pub fn fade_in_blocks(
pub fn fade_out_blocks(
pub fn start_next_round(
pub fn respawn_target_for_round(
pub fn start_match_victory_animation(
pub fn animate_match_victory(
pub fn handle_match_complete(

**Tool Call: grep | head -600 -P "^pub fn start_victory_animation|^pub fn animate_victory|^pub fn check_blocks_settled|^pub fn transition_to_fadeout|^pub fn start_next_round|^pub fn handle_match_complete" /Users/sandro/Development/blocce/src/gameplay/game.rs**
Status: Completed

pub fn check_blocks_settled(
pub fn start_victory_animation(
pub fn animate_victory(
pub fn transition_to_fadeout(
pub fn start_next_round(
pub fn handle_match_complete(

**Tool Call: grep "." /Users/sandro/Development/blocce/src/core/mod.rs**
Status: Completed

// Core game data structures and configuration
pub mod components;
pub mod config;
pub mod fonts;
pub mod player;
pub mod resources;
pub mod states;
// Re-export commonly used types for convenience
pub use components::*;
pub use config::*;
pub use fonts::*;
pub use player::*;
pub use resources::*;
pub use states::*;

**Tool Call: grep "." /Users/sandro/Development/blocce/src/gameplay/mod.rs**
Status: Completed

// Game logic, rules, and arena setup
pub mod arena;
pub mod game;
// Re-export public API
pub use arena::*;
pub use game::*;

**Tool Call: Find `/Users/sandro/Development/blocce/src/ui/**/*.rs`**
Status: Completed

/Users/sandro/Development/blocce/src/ui/game_ui.rs
/Users/sandro/Development/blocce/src/ui/menu/components.rs
/Users/sandro/Development/blocce/src/ui/menu/cleanup.rs
/Users/sandro/Development/blocce/src/ui/menu/mod.rs
/Users/sandro/Development/blocce/src/ui/menu/start_menu.rs
/Users/sandro/Development/blocce/src/ui/menu/player_selection.rs
/Users/sandro/Development/blocce/src/ui/menu/pause_menu.rs
/Users/sandro/Development/blocce/src/ui/menu/countdown.rs
/Users/sandro/Development/blocce/src/ui/menu/turn_notification.rs
/Users/sandro/Development/blocce/src/ui/mod.rs

**Tool Call: grep | head -100 "." /Users/sandro/Development/blocce/src/ui/game_ui.rs**
Status: Completed

// UI setup and update systems
use bevy::prelude::*;
use crate::core::{
    GameConfiguration, GamePhase, GameText, Player1ScoreText, Player2ScoreText, PlayerManager,
};
/// Setup the game UI
pub fn setup_ui(
    mut commands: Commands,
    player_manager: Res<PlayerManager>,
    config: Res<GameConfiguration>,
    fonts: Res<crate::core::GameFonts>,
) {
    let player1 = player_manager.get_player(0).unwrap();
    let player2 = player_manager.get_player(1).unwrap();
    let ui_cfg = &config.ui;
    // UI root
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(ui_cfg.score_margin)),
            ..default()
        })
        .with_children(|parent| {
            // Top row with player scores
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|parent| {
                    // Player 1 score (top left)
                    parent.spawn((
                        Text::new(format!("{} - Score: 0", player1.name)),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: ui_cfg.score_text.size,
                            ..default()
                        },
                        TextColor(player1.color),
                        Player1ScoreText,
                    ));
                    // Center game text
                    parent.spawn((
                        Text::new(format!(
                            "{}'s Turn - Blocks: 0/{}\nP1: -- | P2: --",
                            player1.name, config.game.blocks_per_player
                        )),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: ui_cfg.game_text.size,
                            ..default()
                        },
                        TextColor(ui_cfg.game_text.color()),
                        GameText,
                    ));
                    // Player 2 score (top right)
                    parent.spawn((
                        Text::new(format!("{} - Score: 0", player2.name)),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: ui_cfg.score_text.size,
                            ..default()
                        },
                        TextColor(player2.color),
                        Player2ScoreText,
                        TextLayout {
                            justify: Justify::Right,
                            ..default()
                        },
                    ));
                });
            // Bottom controls text
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Controls: A/D - Aim | Space - Hold/Release to Throw | Up/Down - Move Camera | W/S - Camera Height"),
                        TextFont {
                            font: fonts.ui.clone(),
                            font_size: ui_cfg.controls_text.size,
                            ..default()
                        },
                        TextColor(ui_cfg.controls_text.color()),
                    ));
                });
        });
}
/// Update UI text based on game state
#[allow(clippy::type_complexity)]
pub fn update_ui(

**Tool Call: grep | head -100 -P "pub fn update_ui" /Users/sandro/Development/blocce/src/ui/game_ui.rs**
Status: Failed

```
<tool_use_error>InputValidationError: Grep failed due to the following issue:
An unexpected parameter `A` was provided</tool_use_error>
```

## Assistant

---

Great! Now I have a comprehensive understanding of the game. Let me analyze what a server architecture would look like:

## Server Architecture Analysis for BLOCCE

Based on the codebase exploration, here's my analysis of client vs. server responsibilities:

### Key Observations

**Current State:**
- Pure physics-based, turn-based game (digital bocce)
- Deterministic physics (Avian3D with fixed 12 substeps)
- Turn-based input (hold space to charge throw, A/D to aim)
- Victory determined by closest block to target after all settle
- Best-of-7 match structure

---

## Recommended Architecture: Server-Authoritative with Client Prediction

### Server Side (Authoritative)

**1. Physics Simulation (Critical)**
```
WHY: Prevent cheating on throw velocities, ensure deterministic outcomes
- Run full Avian3D simulation with same config (12 substeps, same physics constants)
- Simulate all block throws with authoritative initial conditions
- Track all block positions, velocities, rotations
- Detect settlement (velocity < 0.3 m/s for 1 second)
- Calculate final distances to target
- Determine round winners
```

**2. Game State Management (Critical)**
```rust
struct ServerGameState {
    current_player: usize,
    blocks_thrown: [usize; 2],
    round_wins: [usize; 2],
    phase: GamePhase,  // Playing, BlocksSettling, VictoryAnimation, etc.
    all_blocks: Vec<BlockState>,  // Position, velocity, settled status
    target_position: Vec3,
    round_winner: Option<usize>,
}
```

**3. Input Validation (Critical)**
```
- Receive client throw commands: { power: f32, rotation: f32 }
- Validate:
  - power ∈ [0.5, 1.0]
  - rotation ∈ [-45°, +45°]
  - throw_count[player] < 3
  - current phase is Playing
  - correct player's turn
- Apply server-side random spin (use seeded RNG for determinism)
- Spawn block with calculate_throw_velocity()
```

**4. Turn Management (Critical)**
```
- Enforce turn order (alternating after each throw)
- Prevent out-of-turn throws
- Track blocks_thrown per player
- Transition to BlocksSettling when all 6 blocks thrown
```

**5. Round/Match Progression (Critical)**
```
- Detect when all blocks settled (1-second threshold)
- Calculate distances: sqrt((block - target)²)
- Determine winner (closest block)
- Update round_wins
- Check match completion (first to 4 wins)
- Trigger VictoryAnimation phase
- Respawn target for next round
```

**6. Anti-Cheat Measures**
```
- Rate limiting on throws (prevent spam)
- Timeout for inactive players
- Server-side collision detection (blocks can't be teleported)
- Hash physics configuration to ensure clients match
```

---

### Client Side (Rendering + Prediction)

**1. Input Collection (Local)**
```
- Capture keyboard input (Space for power, A/D for aim)
- Show local UI feedback (power meter, aim indicator)
- Display throw angle/rotation visually
- Send throw command to server on Space release
```

**2. Client-Side Prediction (Optional but Recommended)**
```
WHY: Reduce perceived latency for local player's throws

Prediction Strategy:
1. Client simulates throw immediately after sending command
2. Run local Avian3D simulation with identical physics config
3. Display predicted block trajectory/settlement
4. When server state arrives, reconcile:
   - If positions match within tolerance (< 5cm), keep prediction
   - If mismatch, smoothly interpolate to server position
```

**3. State Interpolation (Critical for Smooth Visuals)**
```
Server sends snapshots at 20-30 Hz:
{
  tick: u64,
  blocks: [{ id, pos, vel, rot, angular_vel }],
  phase: GamePhase,
  current_player: usize,
}

Client interpolates between snapshots:
- Display position = lerp(last_snapshot, current_snapshot, t)
- Smooth rotation with slerp
- Extrapolate slightly ahead for fluidity
```

**4. Rendering Only**
```
- Camera system (orbital, block-following)
- Lighting (block point lights, directional light)
- Animations (supernova victory effect, fade-in/out)
- UI (scores, turn indicator, controls)
- Bloom/post-processing effects
```

**5. UI Feedback**
```
- Show current player's turn
- Display round scores (0-0, 1-0, etc.)
- Power meter during charging
- Aim indicator (rotation angle)
- "Waiting for blocks to settle..." message
- Victory animations
```

---

### What's Predictable vs. What Needs Authority

**Highly Predictable (Can Run on Client with Validation):**
✅ **Block trajectories after throw**
   - Physics is deterministic with fixed substeps
   - If client has same initial velocity + spin, trajectory matches server
   - **BUT**: Server must validate initial conditions (no velocity hacking)

✅ **Settlement detection**
   - Simple threshold check (velocity < 0.3 for 1 sec)
   - Client can show "settling..." but server makes final call

✅ **Distance calculations**
   - Pure math: `sqrt((x2-x1)² + (y2-y1)² + (z2-z1)²)`
   - Client can show live distances, but server determines winner

**Requires Server Authority (Anti-Cheat Critical):**
🔒 **Throw initial conditions**
   - Power and rotation must be server-validated
   - Random spin must be server-generated (prevent manipulation)

🔒 **Collision resolution**
   - Block-to-block collisions can cause chain reactions
   - Even tiny position differences compound over time
   - Server must be source of truth

🔒 **Out-of-bounds detection**
   - Prevent clients from "rescuing" their own blocks
   - Server marks blocks OutOfBounds

🔒 **Round winner determination**
   - Prevent clients from falsely claiming victory
   - Server calculates distances after settlement

🔒 **Match progression**
   - Round transitions, match completion
   - Server controls all GamePhase transitions

---

### Networking Model

**Recommended: Hybrid Lockstep + Server Authority**

**Why This Works for BLOCCE:**
1. **Turn-based** - Natural synchronization points between throws
2. **Deterministic physics** - Same inputs → same outputs
3. **Low action frequency** - ~6 throws per 2-3 minute round

**Protocol:**

```
Client → Server (on Space release):
{
  type: "Throw",
  power: 0.85,
  rotation: -22.5,
  client_tick: 12345,
}

Server validates → spawns block → begins simulation

Server → All Clients (20 Hz updates):
{
  tick: 12350,
  blocks: [
    { id: 0, player: 0, pos: [1.2, 0.5, -10.3], vel: [0.5, 0, 0.2], ... },
    { id: 1, player: 1, pos: [0.8, 0.5, -9.1], vel: [0.1, 0, 0.05], ... },
  ],
  phase: BlocksSettling,
  settled_blocks: [0],
  current_player: 1,
}

Server → All Clients (on settlement):
{
  type: "RoundComplete",
  distances: [2.3, 1.8],  // Player 0, Player 1
  winner: 1,
  round_wins: [0, 1],
  target_next_round: [4.5, 0, 3.2],
}
```

---

### Collision Management Strategy

**The Challenge:**
- Block-to-block collisions during settling phase
- Collisions can change outcome (knocking opponent's block away)
- Small position deviations snowball over time

**Solutions:**

**Option 1: Server-Only Simulation (Recommended)**
```
✅ Server runs Avian3D, clients receive positions
✅ Clients interpolate between server snapshots
✅ No divergence possible
✅ Latency hidden by smooth interpolation
❌ Requires server to send updates frequently (20-30 Hz)
```

**Option 2: Deterministic Lockstep**
```
✅ Both client and server run identical simulation
✅ Reduced bandwidth (only send inputs + corrections)
✅ Works because physics is deterministic
❌ Requires perfect synchronization (same frame rate, same substeps)
❌ Requires occasional snapshots to prevent drift
❌ More complex to debug
```

**Hybrid Approach (Best for BLOCCE):**
```
1. Server is authoritative, runs full simulation
2. Local player's throw is predicted immediately (feels responsive)
3. Remote player's throws are interpolated from server updates
4. During settling phase, client interpolates server positions
5. Server sends "settling complete" event with final positions
6. Client snaps to final positions if drift occurred
```

---

### Bandwidth Considerations

**Per Block State: ~48 bytes**
```
id: u32           (4 bytes)
player: u8        (1 byte)
position: Vec3    (12 bytes)
velocity: Vec3    (12 bytes)
rotation: Quat    (16 bytes)
angular_vel: Vec3 (12 bytes)
settled: bool     (1 byte)
```

**Per Update (6 blocks active): ~300 bytes**
- 48 bytes × 6 blocks = 288 bytes
- + game state metadata: ~20 bytes
- **Total: ~308 bytes/snapshot**

**At 20 Hz: 6.16 KB/s per client**
- Very reasonable for modern networks
- Could compress rotations (send as euler angles: 12 bytes instead of 16)
- Delta compression: only send changed blocks

---

### Latency Mitigation

**Typical Latency: 50-150ms**

**For Local Player's Throw:**
1. Client predicts immediately (0ms perceived latency)
2. Server validates + simulates (50-150ms later)
3. Client reconciles if prediction was wrong
   - Usually matches within 5cm due to deterministic physics
   - User barely notices correction

**For Remote Player's Throw:**
1. Remote client sends throw command
2. Server receives (50-150ms), validates, simulates
3. Your client receives update (50-150ms)
4. Total: 100-300ms delay
5. Mitigation: Interpolate block position smoothly from spawn point

**During Settling:**
- Interpolation buffers hide latency
- Display position = lerp(t-100ms, t, smoothing_factor)
- Results in smooth visual motion despite 20 Hz updates

---

## Summary Table

| System | Server | Client | Notes |
|--------|--------|--------|-------|
| **Physics Simulation** | ✅ Authoritative | 🔄 Prediction only | Server validates all outcomes |
| **Throw Initial Conditions** | ✅ Validates | ❌ Sends request | Prevents velocity hacking |
| **Random Spin** | ✅ Generates | ❌ Receives | Must be server-seeded |
| **Collision Detection** | ✅ Authoritative | 🔄 Predicted | Server resolves all collisions |
| **Settlement Detection** | ✅ Determines | 🔄 Visual only | Server enforces 1-sec threshold |
| **Distance Calculation** | ✅ Authoritative | 📊 Display only | Server determines winner |
| **Round Transitions** | ✅ Controls | ❌ Receives | Server manages GamePhase |
| **Input Collection** | ❌ None | ✅ Captures | Client validates locally first |
| **Camera/Rendering** | ❌ None | ✅ Local only | Pure visual, no sync needed |
| **UI/Animations** | ❌ None | ✅ Local only | Victory effects triggered by server events |

---

## Recommended Next Steps (If You Were to Implement)

1. **Add networking dependency**: `bevy_renet` or `bevy_quinnet` (both work well with Bevy 0.17)
2. **Create shared protocol crate**: `blocce_protocol` with serializable messages
3. **Refactor `GameState`**: Split into `ServerGameState` and `ClientGameState`
4. **Separate physics systems**: Tag systems with `OnServer` or `OnClient` run conditions
5. **Implement interpolation**: Add `InterpolationBuffer<BlockState>` for smooth visuals
6. **Add prediction**: Clone throw physics for instant local feedback
7. **Test determinism**: Verify same inputs produce same trajectories

The physics determinism is your biggest advantage here - many multiplayer physics games struggle with this, but Avian3D's fixed substeps make it very achievable.

