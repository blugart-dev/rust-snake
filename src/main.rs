//! Entry point: terminal setup, game loop, and input dispatch.

mod constants;
mod game;
mod rendering;
mod snake;
mod theme;

use std::io;
use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::{cursor, execute, terminal};

use constants::{
    CELL_WIDTH, DEFAULT_TICK_MS, DYING_TICK_MS, MAX_BOARD_H, MAX_BOARD_W, MIN_BOARD_H, MIN_BOARD_W,
};
use game::{Game, GameState};
use snake::Direction;
use theme::Theme;

/// A classic Snake game for the terminal.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Board width in logical cells (overrides auto-detection).
    #[arg(long, value_name = "CELLS")]
    width: Option<u16>,

    /// Board height in logical cells (overrides auto-detection).
    #[arg(long, value_name = "CELLS")]
    height: Option<u16>,

    /// Color theme: classic, neon, or monochrome.
    #[arg(long, default_value = "classic")]
    theme: String,

    /// Disable terminal bell sound on food pickup and death.
    #[arg(long)]
    no_bell: bool,
}

/// Computes board dimensions from the terminal size, with optional overrides.
fn compute_board_size(args: &Args) -> (u16, u16) {
    let (term_cols, term_rows) = terminal::size().unwrap_or((80, 24));

    let board_w = args
        .width
        .unwrap_or(term_cols / CELL_WIDTH)
        .clamp(MIN_BOARD_W, MAX_BOARD_W);
    let board_h = args
        .height
        .unwrap_or(term_rows.saturating_sub(1))
        .clamp(MIN_BOARD_H, MAX_BOARD_H);

    (board_w, board_h)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let theme = Theme::from_name(&args.theme).unwrap_or_else(|| {
        eprintln!(
            "Unknown theme '{}'. Available: classic, neon, monochrome",
            args.theme
        );
        std::process::exit(1);
    });

    // Restore the terminal even on panic (raw mode would otherwise leave it unusable).
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
        default_hook(info);
    }));

    let mut stdout = io::stdout();
    let (board_w, board_h) = compute_board_size(&args);
    let mut game = Game::new(board_w, board_h);

    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    // ── Game loop ───────────────────────────────────────────────────────
    let mut last_tick = Instant::now();

    loop {
        rendering::draw(&game, &mut stdout, &theme, !args.no_bell)?;

        // Determine tick interval based on game state
        let tick = match game.state {
            GameState::Playing => game.tick_duration(),
            GameState::Dying(_) => Duration::from_millis(DYING_TICK_MS),
            _ => Duration::from_millis(DEFAULT_TICK_MS),
        };

        // Wait only for the remaining time until the next tick
        let elapsed = last_tick.elapsed();
        let poll_timeout = tick.saturating_sub(elapsed);

        // Process input without advancing the game
        if event::poll(poll_timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,

                    KeyCode::Enter => game.start(),
                    KeyCode::Char('p') => game.toggle_pause(),
                    KeyCode::Char('r') => {
                        let (w, h) = compute_board_size(&args);
                        game.restart(w, h);
                    }

                    KeyCode::Up | KeyCode::Char('w') => game.change_direction(Direction::Up),
                    KeyCode::Down | KeyCode::Char('s') => game.change_direction(Direction::Down),
                    KeyCode::Left | KeyCode::Char('a') => game.change_direction(Direction::Left),
                    KeyCode::Right | KeyCode::Char('d') => game.change_direction(Direction::Right),

                    _ => {}
                },
                Event::Resize(_, _) => {
                    let (w, h) = compute_board_size(&args);
                    match game.state {
                        GameState::Playing => {
                            game.toggle_pause();
                            game.resize(w, h);
                        }
                        GameState::Paused => {
                            game.resize(w, h);
                        }
                        GameState::Menu => game = Game::new(w, h),
                        GameState::Dying(_) | GameState::GameOver | GameState::Win => {}
                    }
                }
                _ => {}
            }
        }

        // Only advance the game when the tick interval has fully elapsed
        if last_tick.elapsed() >= tick {
            game.update();
            last_tick += tick;
            // Prevent catch-up spiral (e.g. after system sleep)
            let now = Instant::now();
            if now.duration_since(last_tick) > tick * 2 {
                last_tick = now;
            }
        }
    }

    // ── Cleanup ─────────────────────────────────────────────────────────
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
