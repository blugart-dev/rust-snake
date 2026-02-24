//! Core game logic: state machine, collision, scoring, and food management.

#[cfg(not(test))]
use std::fs;
#[cfg(not(test))]
use std::path::PathBuf;
use std::collections::HashSet;
use std::time::Duration;

use rand::RngExt;

use crate::constants::{BASE_TICK_MS, MIN_TICK_MS, SPEED_STEP_MS, BONUS_SPAWN_INTERVAL, BONUS_DURATION, BONUS_POINTS};
use crate::snake::{Direction, Position, Snake};

// ── State ───────────────────────────────────────────────────────────────────

/// The current phase of the game.
#[derive(Debug, PartialEq, Eq)]
pub enum GameState {
    /// Title screen shown before the first game.
    Menu,
    /// Active gameplay — snake moves and collisions are checked.
    Playing,
    /// Gameplay frozen; resume with the pause key.
    Paused,
    /// Death flash animation; the `u8` is the current frame (0..=5).
    Dying(u8),
    /// Terminal state after the death animation completes.
    GameOver,
    /// Terminal state when the snake fills the entire board.
    Win,
}

/// Bonus food that appears temporarily for extra points.
pub(crate) struct BonusFood {
    /// Board position of the bonus item.
    pub pos: Position,
    /// Ticks until the bonus despawns. Removed when this reaches 0.
    pub ticks_remaining: u16,
}

/// All game state: snake, food, board, score, and phase.
pub struct Game {
    pub(crate) snake: Snake,
    /// Position of the regular food pellet (always present on the board).
    pub(crate) food: Position,
    /// Optional bonus item; [`None`] when no bonus is active.
    pub(crate) bonus_food: Option<BonusFood>,
    /// Board width in logical cells (including border walls).
    pub(crate) width: u16,
    /// Board height in logical cells (including border walls).
    pub(crate) height: u16,
    pub(crate) state: GameState,
    pub(crate) score: usize,
    /// Persisted across restarts; saved to disk on game over / win.
    pub(crate) high_score: usize,
    /// Ticks elapsed since the last bonus-food spawn attempt.
    ticks_since_bonus: u16,
}

// ── Persistence ─────────────────────────────────────────────────────────────

/// Returns the platform-specific path used to persist the high score.
#[cfg(not(test))]
fn highscore_path() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("snake");
    let _ = fs::create_dir_all(&dir);
    dir.join("highscore.txt")
}

#[cfg(not(test))]
fn load_high_score() -> usize {
    fs::read_to_string(highscore_path())
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

#[cfg(not(test))]
fn save_high_score(score: usize) {
    let _ = fs::write(highscore_path(), score.to_string());
}

#[cfg(test)]
fn load_high_score() -> usize { 0 }

#[cfg(test)]
fn save_high_score(_score: usize) {}

// ── Core logic ──────────────────────────────────────────────────────────────

impl Game {
    /// Creates a new game with the given board dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let mut game = Self {
            snake: Snake::new(width / 2, height / 2),
            food: Position::new(0, 0),
            bonus_food: None,
            width,
            height,
            state: GameState::Menu,
            score: 0,
            high_score: load_high_score(),
            ticks_since_bonus: 0,
        };

        game.spawn_food();
        game
    }

    /// Tick duration decreases as score rises, making the game harder.
    pub fn tick_duration(&self) -> Duration {
        let reduction = (self.score as u64 * SPEED_STEP_MS).min(BASE_TICK_MS - MIN_TICK_MS);
        Duration::from_millis(BASE_TICK_MS - reduction)
    }

    /// Current speed level (1-based).
    pub(crate) fn level(&self) -> u64 {
        let tick_ms = self.tick_duration().as_millis() as u64;
        ((BASE_TICK_MS - tick_ms) / SPEED_STEP_MS) + 1
    }

    /// Transitions from Menu to Playing.
    pub fn start(&mut self) {
        if self.state == GameState::Menu {
            self.state = GameState::Playing;
        }
    }

    /// Collects all free interior cells (not occupied by the snake or `exclude`).
    fn free_cells(&self, exclude: Option<Position>) -> Vec<Position> {
        let occupied: HashSet<Position> = self.snake.body.iter().copied().collect();
        (1..self.width - 1)
            .flat_map(|x| (1..self.height - 1).map(move |y| Position::new(x, y)))
            .filter(|pos| {
                Some(*pos) != exclude && !occupied.contains(pos)
            })
            .collect()
    }

    /// Places food at a random position not occupied by the snake.
    /// Returns `false` if there are no free cells (the player wins).
    fn spawn_food(&mut self) -> bool {
        let inner_cells = ((self.width - 2) as usize) * ((self.height - 2) as usize);
        let occupied = self.snake.body.len();
        if occupied >= inner_cells {
            return false;
        }

        let mut rng = rand::rng();

        if occupied * 2 >= inner_cells {
            // Board is crowded — collect free cells to avoid long random search
            let free = self.free_cells(None);
            self.food = free[rng.random_range(0..free.len())];
        } else {
            let body: HashSet<Position> = self.snake.body.iter().copied().collect();
            loop {
                let pos = Position::new(
                    rng.random_range(1..self.width - 1),
                    rng.random_range(1..self.height - 1),
                );
                if !body.contains(&pos) {
                    self.food = pos;
                    break;
                }
            }
        }
        true
    }

    /// Occasionally spawns or ticks down bonus food.
    fn tick_bonus_food(&mut self) {
        // Tick down existing bonus
        if let Some(ref mut bonus) = self.bonus_food {
            if bonus.ticks_remaining == 0 {
                self.bonus_food = None;
            } else {
                bonus.ticks_remaining -= 1;
            }
            return;
        }

        // Try to spawn new bonus
        self.ticks_since_bonus += 1;
        if self.ticks_since_bonus < BONUS_SPAWN_INTERVAL {
            return;
        }
        self.ticks_since_bonus = 0;

        let mut rng = rand::rng();
        if !rng.random_bool(0.4) {
            return;
        }

        let free = self.free_cells(Some(self.food));
        if free.is_empty() {
            return;
        }
        let pos = free[rng.random_range(0..free.len())];
        self.bonus_food = Some(BonusFood {
            pos,
            ticks_remaining: BONUS_DURATION,
        });
    }

    /// Advances the game by one tick: move, eat, collide.
    pub fn update(&mut self) {
        match self.state {
            GameState::Playing => {}
            GameState::Dying(frame) => {
                if frame >= 5 {
                    self.state = GameState::GameOver;
                } else {
                    self.state = GameState::Dying(frame + 1);
                }
                return;
            }
            _ => return,
        }

        self.snake.apply_queued_direction();

        let head = *self.snake.head();
        let new_head = match self.snake.direction {
            Direction::Up    => Position::new(head.x, head.y.wrapping_sub(1)),
            Direction::Down  => Position::new(head.x, head.y + 1),
            Direction::Left  => Position::new(head.x.wrapping_sub(1), head.y),
            Direction::Right => Position::new(head.x + 1, head.y),
        };

        // Wall collision (checked before inserting head)
        if new_head.x == 0 || new_head.x >= self.width - 1
        || new_head.y == 0 || new_head.y >= self.height - 1 {
            self.snake.body.push_front(new_head);
            self.set_game_over();
            return;
        }

        // Self collision (checked before inserting head, so no skip needed)
        if self.snake.body.contains(&new_head) {
            self.snake.body.push_front(new_head);
            self.set_game_over();
            return;
        }

        self.snake.body.push_front(new_head);

        // Food consumption
        if new_head == self.food {
            self.score += 1;
            if !self.spawn_food() {
                if self.score > self.high_score {
                    self.high_score = self.score;
                    save_high_score(self.high_score);
                }
                self.state = GameState::Win;
                return;
            }
        } else if self.bonus_food.as_ref().is_some_and(|b| b.pos == new_head) {
            self.score += BONUS_POINTS;
            self.bonus_food = None;
        } else {
            self.snake.body.pop_back();
        }

        self.tick_bonus_food();
    }

    /// Begins the death sequence and persists the high score if beaten.
    fn set_game_over(&mut self) {
        self.state = GameState::Dying(0);
        self.bonus_food = None;
        if self.score > self.high_score {
            self.high_score = self.score;
            save_high_score(self.high_score);
        }
    }

    /// Enqueues a direction change (the queue handles 180-degree rejection).
    pub fn change_direction(&mut self, dir: Direction) {
        if self.state == GameState::Playing {
            self.snake.enqueue_direction(dir);
        }
    }

    /// Toggles between Playing and Paused.
    pub fn toggle_pause(&mut self) {
        match self.state {
            GameState::Playing => self.state = GameState::Paused,
            GameState::Paused  => self.state = GameState::Playing,
            _ => {}
        }
    }

    /// Resets the game, preserving the high score.
    pub fn restart(&mut self, width: u16, height: u16) {
        if self.state != GameState::GameOver && self.state != GameState::Win {
            return;
        }
        let high = self.high_score;
        *self = Self::new(width, height);
        self.state = GameState::Playing;
        self.high_score = high;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_game() -> Game {
        let mut game = Game::new(30, 20);
        game.state = GameState::Playing;
        game
    }

    #[test]
    fn new_game_starts_in_menu() {
        let game = Game::new(30, 20);
        assert_eq!(game.state, GameState::Menu);
    }

    #[test]
    fn food_spawns_inside_board() {
        let game = Game::new(30, 20);
        assert!(game.food.x > 0 && game.food.x < 29);
        assert!(game.food.y > 0 && game.food.y < 19);
    }

    #[test]
    fn food_not_on_snake() {
        let game = Game::new(30, 20);
        assert!(!game.snake.body.iter().any(|s| *s == game.food));
    }

    #[test]
    fn direction_reversal_blocked() {
        let mut game = test_game();
        game.change_direction(Direction::Left);
        // Queue rejects the 180-degree reversal entirely
        assert!(game.snake.dir_queue.is_empty());
    }

    #[test]
    fn direction_change_allowed() {
        let mut game = test_game();
        game.change_direction(Direction::Up);
        assert_eq!(game.snake.dir_queue.front(), Some(&Direction::Up));
    }

    #[test]
    fn rapid_direction_changes_queued() {
        let mut game = test_game();
        // Snake faces Right; queue Up then Left (valid L-turn sequence)
        game.change_direction(Direction::Up);
        game.change_direction(Direction::Left);
        assert_eq!(game.snake.dir_queue.len(), 2);
    }

    #[test]
    fn wall_collision_triggers_dying() {
        let mut game = test_game();
        game.snake = Snake::new(15, 1);
        game.snake.direction = Direction::Up;
        game.update();
        assert!(matches!(game.state, GameState::Dying(_)));
    }

    #[test]
    fn eating_food_increases_score() {
        let mut game = test_game();
        let head = *game.snake.head();
        game.food = Position::new(head.x + 1, head.y);
        game.snake.direction = Direction::Right;
        game.update();
        assert_eq!(game.score, 1);
    }

    #[test]
    fn eating_food_grows_snake() {
        let mut game = test_game();
        let head = *game.snake.head();
        game.food = Position::new(head.x + 1, head.y);
        game.snake.direction = Direction::Right;
        let old_len = game.snake.body.len();
        game.update();
        assert_eq!(game.snake.body.len(), old_len + 1);
    }

    #[test]
    fn tick_duration_decreases_with_score() {
        let mut game = test_game();
        let base = game.tick_duration();
        game.score = 10;
        assert!(game.tick_duration() < base);
    }

    #[test]
    fn tick_duration_has_minimum() {
        let mut game = test_game();
        game.score = 1000;
        assert!(game.tick_duration().as_millis() >= MIN_TICK_MS as u128);
    }

    #[test]
    fn high_score_preserved_on_restart() {
        let mut game = test_game();
        game.score = 5;
        game.set_game_over();
        game.state = GameState::GameOver;
        game.restart(30, 20);
        assert_eq!(game.high_score, 5);
        assert_eq!(game.score, 0);
    }

    #[test]
    fn pause_toggle() {
        let mut game = test_game();
        game.toggle_pause();
        assert_eq!(game.state, GameState::Paused);
        game.toggle_pause();
        assert_eq!(game.state, GameState::Playing);
    }

    #[test]
    fn cannot_pause_during_game_over() {
        let mut game = test_game();
        game.state = GameState::GameOver;
        game.toggle_pause();
        assert_eq!(game.state, GameState::GameOver);
    }

    #[test]
    fn self_collision_triggers_dying() {
        let mut game = test_game();
        // Build a U-shape: head at (5,5) facing right, body loops back via (6,5)
        game.snake.body.clear();
        game.snake.body.push_back(Position::new(5, 5));
        game.snake.body.push_back(Position::new(4, 5));
        game.snake.body.push_back(Position::new(4, 6));
        game.snake.body.push_back(Position::new(5, 6));
        game.snake.body.push_back(Position::new(6, 6));
        game.snake.body.push_back(Position::new(6, 5));
        game.snake.direction = Direction::Right;
        game.update();
        assert!(matches!(game.state, GameState::Dying(_)));
    }

    // ── Bonus food tests ───────────────────────────────────────────────

    #[test]
    fn bonus_food_spawns_after_interval() {
        let mut game = test_game();
        // Fast-forward ticks_since_bonus to just before the threshold
        game.ticks_since_bonus = BONUS_SPAWN_INTERVAL - 1;
        // Run many updates to give the 40% random chance a fair shot
        // We place food far away so the snake doesn't eat it
        game.food = Position::new(1, 1);
        game.snake = Snake::new(15, 10);
        let mut bonus_appeared = false;
        for _ in 0..100 {
            game.ticks_since_bonus = BONUS_SPAWN_INTERVAL - 1;
            game.bonus_food = None;
            game.update();
            if game.bonus_food.is_some() {
                bonus_appeared = true;
                break;
            }
        }
        assert!(bonus_appeared, "Bonus food should eventually spawn after interval");
    }

    #[test]
    fn bonus_food_decays_over_ticks() {
        let mut game = test_game();
        game.food = Position::new(1, 1);
        game.snake = Snake::new(15, 10);
        game.bonus_food = Some(BonusFood {
            pos: Position::new(3, 3),
            ticks_remaining: 2,
        });
        game.update(); // ticks_remaining: 2 → 1
        assert!(game.bonus_food.is_some());
        game.update(); // ticks_remaining: 1 → 0
        assert!(game.bonus_food.is_some());
        game.update(); // ticks_remaining == 0, removed
        assert!(game.bonus_food.is_none(), "Bonus food should disappear after its duration");
    }

    #[test]
    fn bonus_food_does_not_overlap_snake_or_food() {
        let mut game = test_game();
        game.ticks_since_bonus = BONUS_SPAWN_INTERVAL - 1;
        game.food = Position::new(1, 1);
        game.snake = Snake::new(15, 10);
        // Run several updates until bonus spawns
        for _ in 0..200 {
            game.ticks_since_bonus = BONUS_SPAWN_INTERVAL - 1;
            game.bonus_food = None;
            game.update();
            if let Some(ref bonus) = game.bonus_food {
                assert_ne!(bonus.pos, game.food, "Bonus should not overlap food");
                assert!(
                    !game.snake.body.iter().any(|seg| *seg == bonus.pos),
                    "Bonus should not overlap snake"
                );
                return;
            }
        }
        panic!("Bonus food never spawned in 200 attempts");
    }

    #[test]
    fn eating_bonus_food_awards_bonus_points() {
        let mut game = test_game();
        let head = *game.snake.head();
        let bonus_pos = Position::new(head.x + 1, head.y);
        game.food = Position::new(1, 1); // far away
        game.bonus_food = Some(BonusFood {
            pos: bonus_pos,
            ticks_remaining: 10,
        });
        game.snake.direction = Direction::Right;
        game.update();
        assert_eq!(game.score, BONUS_POINTS, "Eating bonus food should award BONUS_POINTS");
        assert!(game.bonus_food.is_none(), "Bonus food should be consumed");
    }

    // ── Win condition test ──────────────────────────────────────────────

    #[test]
    fn win_when_board_is_full() {
        // Use a tiny 4×4 board: inner area is 2×2 = 4 cells
        let mut game = Game::new(4, 4);
        game.state = GameState::Playing;

        // Fill the snake to occupy 3 of the 4 inner cells
        game.snake.body.clear();
        game.snake.body.push_back(Position::new(1, 1)); // head
        game.snake.body.push_back(Position::new(2, 1));
        game.snake.body.push_back(Position::new(2, 2));

        // Place food on the last free cell
        game.food = Position::new(1, 2);
        game.snake.direction = Direction::Down;

        game.update();
        assert_eq!(game.state, GameState::Win, "Filling the board should trigger Win");
    }

    // ── Restart rejection tests ────────────────────────────────────────

    #[test]
    fn restart_rejected_during_playing() {
        let mut game = test_game();
        game.restart(30, 20);
        assert_eq!(game.state, GameState::Playing, "Restart should be ignored during Playing");
    }

    #[test]
    fn restart_rejected_during_paused() {
        let mut game = test_game();
        game.state = GameState::Paused;
        game.restart(30, 20);
        assert_eq!(game.state, GameState::Paused, "Restart should be ignored during Paused");
    }

    #[test]
    fn restart_rejected_during_menu() {
        let mut game = Game::new(30, 20);
        game.restart(30, 20);
        assert_eq!(game.state, GameState::Menu, "Restart should be ignored during Menu");
    }

    #[test]
    fn restart_accepted_during_game_over() {
        let mut game = test_game();
        game.state = GameState::GameOver;
        game.restart(30, 20);
        assert_eq!(game.state, GameState::Playing, "Restart should work during GameOver");
        assert_eq!(game.score, 0);
    }

    #[test]
    fn restart_accepted_during_win() {
        let mut game = test_game();
        game.state = GameState::Win;
        game.restart(30, 20);
        assert_eq!(game.state, GameState::Playing, "Restart should work during Win");
        assert_eq!(game.score, 0);
    }

    // ── Start transition tests ─────────────────────────────────────────

    #[test]
    fn start_transitions_menu_to_playing() {
        let mut game = Game::new(30, 20);
        assert_eq!(game.state, GameState::Menu);
        game.start();
        assert_eq!(game.state, GameState::Playing);
    }

    #[test]
    fn start_ignored_during_playing() {
        let mut game = test_game();
        game.start();
        assert_eq!(game.state, GameState::Playing);
    }

    #[test]
    fn start_ignored_during_paused() {
        let mut game = test_game();
        game.state = GameState::Paused;
        game.start();
        assert_eq!(game.state, GameState::Paused);
    }

    #[test]
    fn start_ignored_during_game_over() {
        let mut game = test_game();
        game.state = GameState::GameOver;
        game.start();
        assert_eq!(game.state, GameState::GameOver);
    }

    #[test]
    fn start_ignored_during_win() {
        let mut game = test_game();
        game.state = GameState::Win;
        game.start();
        assert_eq!(game.state, GameState::Win);
    }

    // ── Movement and no-op tests ────────────────────────────────────────

    #[test]
    fn snake_moves_forward_on_normal_tick() {
        let mut game = test_game();
        let head = *game.snake.head();
        // Place food far away so it won't be eaten
        game.food = Position::new(1, 1);
        game.snake.direction = Direction::Right;
        let old_len = game.snake.body.len();
        game.update();
        assert_eq!(*game.snake.head(), Position::new(head.x + 1, head.y));
        assert_eq!(game.snake.body.len(), old_len, "Snake should not grow without eating");
    }

    #[test]
    fn update_noop_during_menu() {
        let mut game = Game::new(30, 20);
        let head = *game.snake.head();
        game.update();
        assert_eq!(game.state, GameState::Menu);
        assert_eq!(*game.snake.head(), head, "Snake should not move during Menu");
    }

    #[test]
    fn update_noop_during_paused() {
        let mut game = test_game();
        let head = *game.snake.head();
        game.state = GameState::Paused;
        game.update();
        assert_eq!(game.state, GameState::Paused);
        assert_eq!(*game.snake.head(), head, "Snake should not move during Paused");
    }
}
