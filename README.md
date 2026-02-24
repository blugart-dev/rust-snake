# rust-snake

A classic Snake game for the terminal, written in Rust.

![Rust](https://img.shields.io/badge/Rust-2024_edition-orange)
[![CI](https://github.com/blugart-dev/rust-snake/actions/workflows/ci.yml/badge.svg)](https://github.com/blugart-dev/rust-snake/actions/workflows/ci.yml)

## Features

- Smooth terminal rendering with Unicode box-drawing and block characters
- Progressive difficulty — the snake speeds up as your score grows
- Bonus food that spawns periodically for extra points
- High score persistence across sessions
- Death flash animation
- Win condition when the board is completely filled
- Responsive board sizing based on terminal dimensions
- Color themes: classic, neon, and monochrome
- CLI flags for board size and theme customization
- Terminal bell feedback on food pickup and death

## Requirements

- Rust 1.85+ (edition 2024)
- A terminal that supports ANSI escape codes and Unicode

## Installation

### From crates.io

```sh
cargo install rust-snake
```

### From source

```sh
cargo run
```

### Pre-built binaries

Download the latest release for your platform from the [Releases](https://github.com/blugart-dev/rust-snake/releases) page. Available for Linux (x86_64), macOS (x86_64, aarch64), and Windows (x86_64).

## Usage

```sh
rust-snake                          # auto-detect board size, classic theme
rust-snake --width 40 --height 25   # custom board dimensions
rust-snake --theme neon             # neon color theme
rust-snake --theme monochrome       # greyscale theme
rust-snake --no-bell                # disable terminal bell sounds
```

## Controls

| Key              | Action       |
| ---------------- | ------------ |
| Arrow keys / WASD | Move        |
| Enter            | Start game   |
| P                | Pause/resume |
| R                | Restart (after game over) |
| Q / Esc          | Quit         |

## Running Tests

```sh
cargo test
```

## Project Structure

```
src/
├── main.rs        # Entry point, terminal setup, game loop, input dispatch
├── game.rs        # Core game logic, state machine, collision, scoring
├── snake.rs       # Snake data types and movement logic
├── rendering.rs   # Terminal rendering for every game state
├── theme.rs       # Color theme definitions (classic, neon, monochrome)
└── constants.rs   # Tunable game parameters
```
