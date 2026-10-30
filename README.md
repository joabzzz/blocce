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

### Run in a browser

The web build uses WebAssembly and WebGL2. Install the Rust target and [Trunk](https://trunkrs.dev/), then run:

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
trunk serve --open
```

For deployment, run `trunk build --release --cargo-profile web-release --locked` and serve the resulting `dist/` directory over HTTP(S), including its `assets/` directory. The `web-release` profile optimizes the WASM binary for size while leaving native release builds unchanged. The relative URLs allow the build to be hosted under a subdirectory. A web app can embed a deployed build using an iframe:

```html
<iframe src="https://example.com/blocce/" title="BLOCCE game"
        style="width: 100%; aspect-ratio: 16 / 9; border: 0"></iframe>
```

Click inside the game to give it keyboard focus. Gameplay currently uses keyboard controls. The default web version omits bloom and motion blur for WebGL2 compatibility. The browser build embeds `config.yaml` at compile time, so rebuild after changing it. The native build reads the file at startup. The game and its assets are subject to their respective licenses; see the license information below and in `assets/`.

### Experimental WebGPU bloom

To try the desktop bloom effect in a browser that supports WebGPU, build a separate package:

```bash
trunk serve --release --features webgpu
# Or produce a package in dist/:
trunk build --release --cargo-profile web-release --locked --features webgpu
```

This selects WebGPU instead of WebGL2 and does not fall back automatically on browsers without WebGPU. Use the default build for wider browser support. Motion blur remains disabled in both web builds.

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

Copyright (C) 2026 Alessandro Joabar

BLOCCE's project code is available under the [PolyForm Noncommercial License
1.0.0](LICENSE). It permits use, modification, and distribution under its
terms, including for noncommercial purposes. See the license for its full
terms and permitted uses.

Third-party components have their own licenses. The Inter fonts are licensed
under the [SIL Open Font License](assets/Inter/OFL.txt), as is
[Silkscreen](assets/Silkscreen-OFL.txt). Rust dependencies are subject to
their respective licenses.

