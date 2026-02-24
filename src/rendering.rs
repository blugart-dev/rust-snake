//! Terminal rendering for every game state.
//!
//! All draw commands are queued into a single buffer via [`crossterm::queue!`]
//! and flushed once per frame in [`draw`] to eliminate flickering.

use std::io::{self, Write};

use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use unicode_width::UnicodeWidthStr;

use crate::constants::CELL_WIDTH;
use crate::game::{Game, GameState};
use crate::theme::Theme;

// ── Box-drawing characters ──────────────────────────────────────────────────

const WALL_TL: &str = "╔";
const WALL_TR: &str = "╗";
const WALL_BL: &str = "╚";
const WALL_BR: &str = "╝";
const WALL_H: &str = "═";
const WALL_V: &str = "║";

// ── Sprite characters ──────────────────────────────────────────────────────

const SNAKE_HEAD: &str = "█";
const SNAKE_BODY: &str = "▓";
const FOOD_CHAR: &str = "●";
const BONUS_CHAR: &str = "★";

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Converts a logical game cell to a terminal cursor column and row.
fn cell_to_terminal(x: u16, y: u16) -> (u16, u16) {
    (x * CELL_WIDTH, y)
}

/// Returns the total terminal columns spanned by the board.
fn terminal_width(board_width: u16) -> u16 {
    board_width * CELL_WIDTH
}

// ── Public entry point ─────────────────────────────────────────────────────

/// Renders the full game frame. All draw commands are queued into a single
/// buffer and flushed once at the end to prevent flickering.
pub fn draw(game: &Game, w: &mut impl Write, theme: &Theme) -> io::Result<()> {
    draw_border(game, w, theme)?;
    draw_interior(game, w)?;

    match game.state {
        GameState::Menu => {
            draw_menu(game, w, theme)?;
        }
        _ => {
            draw_food(game, w, theme)?;
            draw_snake(game, w, theme)?;
            draw_status_bar(game, w, theme)?;

            match game.state {
                GameState::Paused => {
                    draw_overlay(game, w, &[("--- PAUSED ---".into(), Color::Yellow)])?;
                }
                GameState::Dying(frame) => {
                    draw_snake_death(game, w, frame % 2 == 0)?;
                }
                GameState::GameOver => {
                    let is_new_high = game.score > 0 && game.score >= game.high_score;
                    let mut lines: Vec<(String, Color)> = vec![
                        ("GAME OVER".into(), Color::Red),
                        (String::new(), Color::White),
                    ];
                    if is_new_high {
                        lines.push(("NEW HIGH SCORE!".into(), Color::Yellow));
                        lines.push((String::new(), Color::White));
                    }
                    lines.push((
                        format!(
                            "Score: {}  Length: {}  Level: {}",
                            game.score,
                            game.snake.body.len(),
                            game.level()
                        ),
                        Color::Yellow,
                    ));
                    lines.push((String::new(), Color::White));
                    lines.push(("[R] Play Again".into(), Color::White));
                    lines.push(("[Q] Quit".into(), Color::DarkGrey));
                    draw_overlay(game, w, &lines)?;
                }
                GameState::Win => {
                    let mut lines: Vec<(String, Color)> = vec![
                        ("YOU WIN!".into(), Color::Green),
                        (String::new(), Color::White),
                        (
                            format!(
                                "Score: {}  Length: {}  Level: {}",
                                game.score,
                                game.snake.body.len(),
                                game.level()
                            ),
                            Color::Yellow,
                        ),
                        (String::new(), Color::White),
                        ("[R] Play Again".into(), Color::White),
                        ("[Q] Quit".into(), Color::DarkGrey),
                    ];
                    if game.score > 0 && game.score >= game.high_score {
                        lines.insert(2, ("NEW HIGH SCORE!".into(), Color::Yellow));
                        lines.insert(3, (String::new(), Color::White));
                    }
                    draw_overlay(game, w, &lines)?;
                }
                _ => {}
            }
        }
    }

    queue!(w, ResetColor)?;
    w.flush()?;
    Ok(())
}

// ── Internal draw helpers ──────────────────────────────────────────────────

/// Draws the double-line box border around the board.
fn draw_border(game: &Game, w: &mut impl Write, theme: &Theme) -> io::Result<()> {
    let tw = terminal_width(game.width);
    // Each corner occupies 1 column; the remaining (CELL_WIDTH - 1) columns
    // of the wall cells are filled with horizontal wall segments.
    let wall_pad = WALL_H.repeat((CELL_WIDTH - 1) as usize);
    let h_cell = WALL_H.repeat(CELL_WIDTH as usize);
    let h_inner = h_cell.repeat((game.width - 2) as usize);

    queue!(w, SetForegroundColor(theme.wall))?;

    queue!(
        w,
        cursor::MoveTo(0, 0),
        Print(format!("{WALL_TL}{wall_pad}{h_inner}{wall_pad}{WALL_TR}")),
    )?;

    queue!(
        w,
        cursor::MoveTo(0, game.height - 1),
        Print(format!("{WALL_BL}{wall_pad}{h_inner}{wall_pad}{WALL_BR}")),
    )?;

    for y in 1..game.height - 1 {
        queue!(w, cursor::MoveTo(0, y), Print(WALL_V))?;
        queue!(w, cursor::MoveTo(tw - 1, y), Print(WALL_V))?;
    }

    Ok(())
}

/// Clears the playable interior with spaces (erases previous frame).
fn draw_interior(game: &Game, w: &mut impl Write) -> io::Result<()> {
    let tw = terminal_width(game.width);
    let row = " ".repeat((tw - 2) as usize);
    for y in 1..game.height - 1 {
        queue!(w, cursor::MoveTo(1, y), Print(&row))?;
    }
    Ok(())
}

/// Draws the regular food pellet and, if active, the bonus food item.
fn draw_food(game: &Game, w: &mut impl Write, theme: &Theme) -> io::Result<()> {
    let (tx, ty) = cell_to_terminal(game.food.x, game.food.y);
    queue!(
        w,
        cursor::MoveTo(tx, ty),
        SetForegroundColor(theme.food),
        Print(FOOD_CHAR),
        Print(FOOD_CHAR),
    )?;

    if let Some(ref bonus) = game.bonus_food {
        let (tx, ty) = cell_to_terminal(bonus.pos.x, bonus.pos.y);
        queue!(
            w,
            cursor::MoveTo(tx, ty),
            SetForegroundColor(theme.bonus),
            Print(BONUS_CHAR),
            Print(BONUS_CHAR),
        )?;
    }

    Ok(())
}

/// Draws every segment of the snake (head in a distinct color).
fn draw_snake(game: &Game, w: &mut impl Write, theme: &Theme) -> io::Result<()> {
    for (i, seg) in game.snake.body.iter().enumerate() {
        let (ch, color) = if i == 0 {
            (SNAKE_HEAD, theme.head)
        } else {
            (SNAKE_BODY, theme.body)
        };
        let (tx, ty) = cell_to_terminal(seg.x, seg.y);
        queue!(
            w,
            cursor::MoveTo(tx, ty),
            SetForegroundColor(color),
            Print(ch),
            Print(ch),
        )?;
    }
    Ok(())
}

/// Redraws the snake in alternating red tones for the death flash effect.
fn draw_snake_death(game: &Game, w: &mut impl Write, flash: bool) -> io::Result<()> {
    let color = if flash { Color::Red } else { Color::DarkRed };
    for seg in game.snake.body.iter() {
        let (tx, ty) = cell_to_terminal(seg.x, seg.y);
        queue!(
            w,
            cursor::MoveTo(tx, ty),
            SetForegroundColor(color),
            Print(SNAKE_BODY),
            Print(SNAKE_BODY),
        )?;
    }
    Ok(())
}

/// Draws the HUD row below the board (score, high score, level, controls).
fn draw_status_bar(game: &Game, w: &mut impl Write, theme: &Theme) -> io::Result<()> {
    let tw = terminal_width(game.width);
    let clear = " ".repeat(tw as usize);
    queue!(w, cursor::MoveTo(0, game.height), Print(&clear))?;

    let level = game.level();
    queue!(
        w,
        cursor::MoveTo(0, game.height),
        SetForegroundColor(theme.status_score),
        Print(format!(" Score: {} ", game.score)),
        SetForegroundColor(Color::DarkGrey),
        Print("·"),
        SetForegroundColor(theme.status_best),
        Print(format!(" Best: {} ", game.high_score)),
        SetForegroundColor(Color::DarkGrey),
        Print("·"),
        SetForegroundColor(theme.status_level),
        Print(format!(" Lv.{level} ")),
        SetForegroundColor(Color::DarkGrey),
        Print("·"),
        SetForegroundColor(Color::White),
        Print(" P:Pause "),
        SetForegroundColor(Color::DarkGrey),
        Print("·"),
        SetForegroundColor(Color::White),
        Print(" Q:Quit"),
    )?;
    Ok(())
}

/// Renders centered text lines over the board (pause, game over, win screens).
fn draw_overlay(game: &Game, w: &mut impl Write, lines: &[(String, Color)]) -> io::Result<()> {
    let tw = terminal_width(game.width);
    let center_x = tw / 2;
    let start_y = game.height / 2 - lines.len() as u16 / 2;
    for (i, (text, color)) in lines.iter().enumerate() {
        let x = center_x.saturating_sub(text.width() as u16 / 2);
        queue!(
            w,
            cursor::MoveTo(x, start_y + i as u16),
            SetForegroundColor(*color),
            Print(text),
        )?;
    }
    Ok(())
}

/// Draws the title screen with ASCII art, controls, and high score.
fn draw_menu(game: &Game, w: &mut impl Write, theme: &Theme) -> io::Result<()> {
    let tw = terminal_width(game.width);
    let cx = tw / 2;

    let title = [
        r" ___  _  _   _   _  _ ___ ",
        r"/ __|| \| | / \ | |/ / __|",
        r"\__ \| .` |/ _ \|   <| _| ",
        r"|___/|_|\_/_/ \_\_|\_\___|",
    ];

    let title_y = game.height / 5;
    queue!(w, SetForegroundColor(theme.title))?;
    for (i, line) in title.iter().enumerate() {
        let x = cx.saturating_sub(line.width() as u16 / 2);
        queue!(w, cursor::MoveTo(x, title_y + i as u16), Print(line))?;
    }

    let info: Vec<(&str, Color)> = vec![
        ("Arrow Keys / WASD to move", Color::White),
        ("P: Pause   Q/Esc: Quit", Color::DarkGrey),
        ("", Color::White),
        ("Press ENTER to start", Color::Yellow),
    ];

    let info_y = title_y + title.len() as u16 + 3;
    for (i, (text, color)) in info.iter().enumerate() {
        let x = cx.saturating_sub(text.width() as u16 / 2);
        queue!(
            w,
            cursor::MoveTo(x, info_y + i as u16),
            SetForegroundColor(*color),
            Print(text),
        )?;
    }

    if game.high_score > 0 {
        let hs = format!("High Score: {}", game.high_score);
        let x = cx.saturating_sub(hs.width() as u16 / 2);
        queue!(
            w,
            cursor::MoveTo(x, info_y + info.len() as u16 + 1),
            SetForegroundColor(theme.status_best),
            Print(hs),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;

    fn render_to_string(game: &Game) -> String {
        let theme = Theme::classic();
        let mut buf = Vec::new();
        draw(game, &mut buf, &theme).unwrap();
        String::from_utf8_lossy(&buf).to_string()
    }

    #[test]
    fn menu_render_contains_snake_title() {
        let game = Game::new(30, 20);
        let output = render_to_string(&game);
        // The title is ASCII art; check for a recognizable fragment
        assert!(
            output.contains("___"),
            "Menu should contain ASCII art title"
        );
    }

    #[test]
    fn menu_render_contains_start_prompt() {
        let game = Game::new(30, 20);
        let output = render_to_string(&game);
        assert!(
            output.contains("ENTER"),
            "Menu should prompt to press ENTER"
        );
    }

    #[test]
    fn playing_render_contains_score() {
        let mut game = Game::new(30, 20);
        game.state = GameState::Playing;
        let output = render_to_string(&game);
        assert!(output.contains("Score:"), "Playing state should show score");
    }

    #[test]
    fn paused_render_contains_paused_text() {
        let mut game = Game::new(30, 20);
        game.state = GameState::Paused;
        let output = render_to_string(&game);
        assert!(output.contains("PAUSED"), "Paused state should show PAUSED");
    }

    #[test]
    fn game_over_render_contains_game_over_text() {
        let mut game = Game::new(30, 20);
        game.state = GameState::GameOver;
        let output = render_to_string(&game);
        assert!(
            output.contains("GAME OVER"),
            "GameOver state should show GAME OVER"
        );
    }

    #[test]
    fn win_render_contains_win_text() {
        let mut game = Game::new(30, 20);
        game.state = GameState::Win;
        let output = render_to_string(&game);
        assert!(output.contains("YOU WIN"), "Win state should show YOU WIN");
    }
}
