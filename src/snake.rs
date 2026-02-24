//! Snake data types and movement logic.

use std::collections::VecDeque;

/// The four cardinal directions the snake can move.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns the opposite direction.
    pub fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// A 2D coordinate on the game board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// The snake: a deque of body segments, a movement direction, and a small
/// input queue so rapid keypresses between ticks aren't lost.
///
/// Uses `VecDeque` for O(1) push_front (new head) and pop_back (tail removal).
pub struct Snake {
    /// Ordered segments: index 0 is the head, last element is the tail.
    pub body: VecDeque<Position>,
    /// Current movement direction applied on the next tick.
    pub direction: Direction,
    /// Buffered direction changes from keypresses between ticks.
    pub dir_queue: VecDeque<Direction>,
}

/// Number of body segments the snake starts with.
pub const INITIAL_LENGTH: u16 = 3;
/// Maximum buffered direction changes to prevent input flooding.
const MAX_DIR_QUEUE: usize = 2;

impl Snake {
    /// Creates a snake of length 3, centered at `(x, y)`, facing right.
    pub fn new(x: u16, y: u16) -> Self {
        let body = (0..INITIAL_LENGTH)
            .map(|i| Position::new(x - i, y))
            .collect();

        Self {
            body,
            direction: Direction::Right,
            dir_queue: VecDeque::new(),
        }
    }

    /// Returns a reference to the head position.
    pub fn head(&self) -> &Position {
        &self.body[0]
    }

    /// Enqueues a direction change, ignoring 180-degree reversals relative to
    /// the last queued (or current) direction.
    pub fn enqueue_direction(&mut self, dir: Direction) {
        let effective = self.dir_queue.back().copied().unwrap_or(self.direction);
        if effective.opposite() != dir && effective != dir && self.dir_queue.len() < MAX_DIR_QUEUE {
            self.dir_queue.push_back(dir);
        }
    }

    /// Pops the next queued direction (if any) and applies it.
    pub fn apply_queued_direction(&mut self) {
        if let Some(dir) = self.dir_queue.pop_front() {
            self.direction = dir;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_initial_length() {
        let snake = Snake::new(10, 10);
        assert_eq!(snake.body.len(), INITIAL_LENGTH as usize);
    }

    #[test]
    fn snake_head_position() {
        let snake = Snake::new(10, 10);
        assert_eq!(*snake.head(), Position::new(10, 10));
    }

    #[test]
    fn snake_initial_direction() {
        let snake = Snake::new(10, 10);
        assert_eq!(snake.direction, Direction::Right);
    }

    #[test]
    fn direction_opposite() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Down.opposite(), Direction::Up);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Right.opposite(), Direction::Left);
    }

    #[test]
    fn direction_not_self_opposite() {
        assert_ne!(Direction::Up.opposite(), Direction::Up);
        assert_ne!(Direction::Left.opposite(), Direction::Left);
    }
}
