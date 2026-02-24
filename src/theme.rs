//! Color themes for the game.

use crossterm::style::Color;

/// A complete color palette for the game.
pub struct Theme {
    pub wall: Color,
    pub head: Color,
    pub body: Color,
    pub food: Color,
    pub bonus: Color,
    pub title: Color,
    pub status_score: Color,
    pub status_best: Color,
    pub status_level: Color,
}

impl Theme {
    /// The default green-on-dark palette matching the original game.
    pub fn classic() -> Self {
        Self {
            wall: Color::DarkGrey,
            head: Color::Green,
            body: Color::DarkGreen,
            food: Color::Red,
            bonus: Color::Magenta,
            title: Color::Green,
            status_score: Color::Yellow,
            status_best: Color::Magenta,
            status_level: Color::Cyan,
        }
    }

    /// Bright, vibrant colors for a neon arcade look.
    pub fn neon() -> Self {
        Self {
            wall: Color::White,
            head: Color::Cyan,
            body: Color::Blue,
            food: Color::Yellow,
            bonus: Color::Magenta,
            title: Color::Cyan,
            status_score: Color::Yellow,
            status_best: Color::Magenta,
            status_level: Color::Cyan,
        }
    }

    /// Greyscale palette for minimal terminals.
    pub fn monochrome() -> Self {
        Self {
            wall: Color::DarkGrey,
            head: Color::White,
            body: Color::Grey,
            food: Color::White,
            bonus: Color::Grey,
            title: Color::White,
            status_score: Color::White,
            status_best: Color::Grey,
            status_level: Color::White,
        }
    }

    /// Resolves a theme name to a palette. Returns `None` for unknown names.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "classic" => Some(Self::classic()),
            "neon" => Some(Self::neon()),
            "monochrome" => Some(Self::monochrome()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_name_classic() {
        assert!(Theme::from_name("classic").is_some());
    }

    #[test]
    fn from_name_neon() {
        assert!(Theme::from_name("neon").is_some());
    }

    #[test]
    fn from_name_monochrome() {
        assert!(Theme::from_name("monochrome").is_some());
    }

    #[test]
    fn from_name_invalid() {
        assert!(Theme::from_name("rainbow").is_none());
    }
}
