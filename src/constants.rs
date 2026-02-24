//! Tunable game parameters.
//!
//! All magic numbers live here so gameplay can be tweaked from a single file.

// ── Board sizing ────────────────────────────────────────────────────────────

/// Minimum board width in logical cells.
pub const MIN_BOARD_W: u16 = 20;
/// Maximum board width in logical cells.
pub const MAX_BOARD_W: u16 = 50;
/// Minimum board height in logical cells.
pub const MIN_BOARD_H: u16 = 15;
/// Maximum board height in logical cells.
pub const MAX_BOARD_H: u16 = 35;

// ── Rendering ───────────────────────────────────────────────────────────────

/// Terminal columns per logical cell (2 columns ≈ square aspect ratio).
pub const CELL_WIDTH: u16 = 2;

// ── Speed curve ─────────────────────────────────────────────────────────────
//
// Tick interval = BASE_TICK_MS − (score × SPEED_STEP_MS), clamped to MIN_TICK_MS.

/// Starting tick interval (ms) at score 0.
pub const BASE_TICK_MS: u64 = 200;
/// Fastest allowed tick interval (ms).
pub const MIN_TICK_MS: u64 = 60;
/// Tick reduction per point scored (ms).
pub const SPEED_STEP_MS: u64 = 5;

// ── Tick intervals for non-playing states ───────────────────────────────────

/// Tick interval (ms) during the death flash animation.
pub const DYING_TICK_MS: u64 = 150;
/// Tick interval (ms) for menu, pause, and other idle states.
pub const DEFAULT_TICK_MS: u64 = 100;

// ── Bonus food ──────────────────────────────────────────────────────────────

/// Number of ticks between bonus-food spawn attempts.
pub const BONUS_SPAWN_INTERVAL: u16 = 30;
/// How many ticks bonus food stays on the board before expiring.
pub const BONUS_DURATION: u16 = 20;
/// Points awarded for eating bonus food.
pub const BONUS_POINTS: usize = 3;
