# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

- **Build:** `cargo build`
- **Run:** `cargo run` (runs in an alternate terminal screen with raw mode)
- **Run with flags:** `cargo run -- --width 40 --height 25 --theme neon --no-bell`
- **Test all:** `cargo test`
- **Test single:** `cargo test <test_name>` (e.g., `cargo test wall_collision_triggers_dying`)
- **Check (no codegen):** `cargo check`
- **Clippy:** `cargo clippy`
- **Format check:** `cargo fmt --check`

## Architecture

Terminal-based Snake game written in Rust (edition 2024) using crossterm for terminal I/O.

### Module Overview

- **`main.rs`** — Game loop, terminal setup/teardown (raw mode, alternate screen), input handling via crossterm events. Parses CLI arguments via `clap` (`--width`, `--height`, `--theme`, `--no-bell`). Computes board dimensions from terminal size with optional overrides. Restores terminal on panic.
- **`game.rs`** — Core game logic and state machine (`Game` struct). Manages snake movement, food spawning, collision detection, scoring, bonus food, speed progression, board resizing, and high score persistence (saved to `dirs::data_local_dir()/snake/highscore.txt`). High score I/O is stubbed out in `#[cfg(test)]`. Exposes transient `ate_food`/`ate_bonus`/`just_died` flags for audio feedback.
- **`snake.rs`** — `Snake` struct (VecDeque-based body), `Position`, `Direction` types. Implements a 2-element direction queue to buffer rapid keypresses between ticks and reject 180-degree reversals.
- **`rendering.rs`** — All terminal drawing. Uses queued crossterm commands flushed once per frame to prevent flickering. Renders border, snake, food, bonus food, status bar, overlays (pause/game over/win), and ASCII art menu. Accepts a `Theme` for color customization and optional terminal bell.
- **`theme.rs`** — Color theme definitions. `Theme` struct holds all game colors; built-in palettes: `classic` (default green-on-dark), `neon` (bright cyan/blue/yellow), `monochrome` (greyscale).
- **`constants.rs`** — All tunable game parameters: board size limits, cell width, speed curve, tick intervals, bonus food timing/points.

### Game State Machine

`GameState` enum: `Menu → Playing ⇄ Paused`, `Playing → Dying(frame) → GameOver`, `Playing → Win`. Restart is only allowed from `GameOver` or `Win`.

### Key Design Patterns

- Each logical cell is 2 terminal columns wide (`CELL_WIDTH = 2`) to approximate square cells.
- Speed increases with score: tick duration = `BASE_TICK_MS - (score * SPEED_STEP_MS)`, clamped to `MIN_TICK_MS`.
- Tick timing carries overshoot between frames to prevent drift, with a safety clamp to prevent catch-up spiral.
- Food spawning uses random rejection sampling when the board is sparse, switches to collecting free cells when >50% occupied.
- Bonus food spawns probabilistically (40% chance) every `BONUS_SPAWN_INTERVAL` ticks, lasts `BONUS_DURATION` ticks.
- The `Dying` state plays a 6-frame red flash animation before transitioning to `GameOver`.
- Terminal resize during gameplay attempts to adapt the board; triggers death if the snake doesn't fit.

### CI/CD

- **CI** (`.github/workflows/ci.yml`): Runs on every push to `main` and all PRs. Executes `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` across Ubuntu, macOS, and Windows.
- **Release** (`.github/workflows/release.yml`): Triggered by `v*` tags. Builds release binaries for 4 targets (`x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`), packages them with LICENSE and README, and publishes a GitHub Release with all archives.
