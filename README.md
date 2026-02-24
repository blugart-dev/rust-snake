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

## Requirements

- Rust 1.85+ (edition 2024)
- A terminal that supports ANSI escape codes and Unicode

## Installation

### From source

```sh
cargo run
```

### Pre-built binaries

Download the latest release for your platform from the [Releases](https://github.com/blugart-dev/rust-snake/releases) page. Available for Linux (x86_64), macOS (x86_64, aarch64), and Windows (x86_64).

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
└── constants.rs   # Tunable game parameters
```
