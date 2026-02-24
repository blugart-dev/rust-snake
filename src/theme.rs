//! Color themes for the game.

use clap::ValueEnum;
use crossterm::style::Color;

/// Available color theme names, parsed directly by clap.
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum ThemeName {
    /// Green-on-dark palette (the original).
    #[default]
    Classic,
    /// Bright cyan/blue/yellow arcade colors.
    Neon,
    /// Greyscale for minimal terminals.
    Monochrome,
}

/// A complete color palette for the game.
pub struct Theme {
    pub wall: Color,
    pub head: Color,
    pub body: Color,
    pub food: Color,
    pub bonus: Color,
    pub title: Color,
    pub death_flash: Color,
    pub death_dark: Color,
    pub status_score: Color,
    pub status_best: Color,
    pub status_level: Color,
}

impl Theme {
    /// Builds a [`Theme`] from the CLI-selected name.
    pub fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Classic => Self::classic(),
            ThemeName::Neon => Self::neon(),
            ThemeName::Monochrome => Self::monochrome(),
        }
    }

    fn classic() -> Self {
        Self {
            wall: Color::DarkGrey,
            head: Color::Green,
            body: Color::DarkGreen,
            food: Color::Red,
            bonus: Color::Magenta,
            title: Color::Green,
            death_flash: Color::Red,
            death_dark: Color::DarkRed,
            status_score: Color::Yellow,
            status_best: Color::Magenta,
            status_level: Color::Cyan,
        }
    }

    fn neon() -> Self {
        Self {
            wall: Color::White,
            head: Color::Cyan,
            body: Color::Blue,
            food: Color::Yellow,
            bonus: Color::Magenta,
            title: Color::Cyan,
            death_flash: Color::Red,
            death_dark: Color::DarkRed,
            status_score: Color::Yellow,
            status_best: Color::Magenta,
            status_level: Color::Cyan,
        }
    }

    fn monochrome() -> Self {
        Self {
            wall: Color::DarkGrey,
            head: Color::White,
            body: Color::Grey,
            food: Color::White,
            bonus: Color::Grey,
            title: Color::White,
            death_flash: Color::White,
            death_dark: Color::DarkGrey,
            status_score: Color::White,
            status_best: Color::Grey,
            status_level: Color::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_theme_names_produce_a_theme() {
        for name in ThemeName::value_variants() {
            let theme = Theme::from_name(*name);
            // Smoke-check: every theme has distinct head vs body colors
            assert_ne!(
                format!("{:?}", theme.head),
                format!("{:?}", theme.body),
                "{name:?} should have distinct head/body colors"
            );
        }
    }
}
