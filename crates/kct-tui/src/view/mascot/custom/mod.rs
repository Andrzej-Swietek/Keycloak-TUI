mod decode;
mod load;

use std::sync::Arc;

use ratatui::style::Color;

pub use load::{MASCOT_DIR_ENV, load_from, mascot_dir};

/// Read the art in [`mascot_dir`]. Returns an empty [`Scan`] when the directory is absent.
#[must_use]
pub fn scan() -> Scan {
    mascot_dir().map(|dir| load_from(&dir)).unwrap_or_default()
}

use super::Mood;

/// Largest art [`load_from`] accepts, in pixels.
pub const MAX_WIDTH: usize = 48;
pub const MAX_HEIGHT: usize = 40;

/// One drawn frame: transparent pixels are `None` and stay the terminal's own background.
#[derive(Debug, Clone)]
pub struct Art {
    pub width: usize,
    pub pixels: Arc<[Option<Color>]>,
}

impl Art {
    #[must_use]
    pub fn height(&self) -> usize {
        self.pixels.len().checked_div(self.width).unwrap_or(0)
    }

    #[must_use]
    pub fn row(&self, index: usize) -> &[Option<Color>] {
        let start = index * self.width;
        self.pixels
            .get(start..start + self.width)
            .unwrap_or_default()
    }

    /// First row with anything drawn, rounded down to an even row so the half-block pairing
    /// keeps its alignment.
    #[must_use]
    pub fn top(&self) -> usize {
        (0..self.height())
            .find(|index| self.row(*index).iter().any(Option::is_some))
            .unwrap_or(0)
            & !1
    }
}

/// A character loaded from image files.
///
/// A mood with no frame of its own renders `default`.
#[derive(Debug, Clone)]
pub struct CustomMascot {
    pub name: Arc<str>,
    pub default: Art,
    pub moods: Arc<[(Mood, Art)]>,
}

impl CustomMascot {
    #[must_use]
    pub fn frame(&self, mood: Mood) -> &Art {
        self.moods
            .iter()
            .find(|(drawn, _)| *drawn == mood)
            .map_or(&self.default, |(_, art)| art)
    }
}

/// What a scan found, including why anything was skipped: a mascot that silently fails to
/// appear is worse than one that says it was the wrong size.
#[derive(Debug, Clone, Default)]
pub struct Scan {
    pub found: Arc<[CustomMascot]>,
    pub problems: Arc<[String]>,
}

#[cfg(test)]
mod tests;
