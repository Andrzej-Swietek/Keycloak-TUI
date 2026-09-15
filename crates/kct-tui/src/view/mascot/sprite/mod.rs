//! The sprite registry.
//!
//! Every design is the same size with the same face geometry, so an expression written once
//! works on all of them and adding a character is one file plus one line here.

mod cat;
mod expression;
mod ghost;
mod miku;
mod robot;
mod twintails;

pub use super::mood::Mood;

/// Sprite size in pixels. Each text cell holds two vertical pixels.
pub const WIDTH: usize = 25;
pub const HEIGHT: usize = 20;

/// Where the expression is patched in. The column and the gap between eyes and mouth are
/// fixed, so an expression written once works on every design; only the height varies, which
/// is what lets a character wear its head high and keep the rows below for hair or shoulders.
pub const EYE_ROWS: usize = 3;
const MOUTH_BELOW_EYES: usize = 4;

/// In an expression, "leave whatever the body has here alone".
const KEEP: char = ' ';

/// The iris an expression is written with. Each design recolours it to its own.
const IRIS: char = 'e';

/// One character: a static body plus how it emotes.
#[derive(Debug)]
pub struct Design {
    pub name: &'static str,
    pub body: [&'static str; HEIGHT],
    /// Palette key this character's irises are painted with.
    pub iris: char,
    /// Row the eyes start on. The mouth follows `MOUTH_BELOW_EYES` under it.
    pub eye_top: usize,
    /// Column the expression is written from.
    pub face_at: usize,
    pub eyes: fn(Mood, u64) -> [&'static str; EYE_ROWS],
    pub mouth: fn(Mood) -> &'static str,
}

impl Design {
    pub const fn mouth_row(&self) -> usize {
        self.eye_top + MOUTH_BELOW_EYES
    }

    /// First drawn row, so a bust gets a short panel and a full figure a tall one.
    ///
    /// Rounded down to an even row: two pixels share a cell, and starting on an odd one would
    /// pair every pixel with the wrong neighbour.
    #[must_use]
    pub fn top(&self) -> usize {
        self.body
            .iter()
            .position(|row| {
                row.chars()
                    .any(|pixel| pixel != super::palette::TRANSPARENT)
            })
            .unwrap_or(0)
            & !1
    }
}

/// Who is in the corner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Character {
    #[default]
    Twintails,
    Miku,
    Cat,
    Robot,
    Ghost,
}

impl Character {
    pub const ALL: [Self; 5] = [
        Self::Twintails,
        Self::Miku,
        Self::Cat,
        Self::Robot,
        Self::Ghost,
    ];

    #[must_use]
    pub const fn design(self) -> &'static Design {
        match self {
            Self::Twintails => &twintails::DESIGN,
            Self::Miku => &miku::DESIGN,
            Self::Cat => &cat::DESIGN,
            Self::Robot => &robot::DESIGN,
            Self::Ghost => &ghost::DESIGN,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        self.design().name
    }

    /// Parse a name from the command line or the config file.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let wanted = raw.trim().to_ascii_lowercase();
        Self::ALL
            .into_iter()
            .find(|character| character.name() == wanted)
    }

    #[must_use]
    pub fn next(self) -> Self {
        let position = Self::ALL
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or(0);
        Self::ALL[(position + 1) % Self::ALL.len()]
    }
}

/// The sprite for this character and mood at this point in the animation.
#[must_use]
pub fn frame(character: Character, mood: Mood, tick: u64) -> [String; HEIGHT] {
    let design = character.design();
    let mut rows: Vec<String> = design.body.iter().map(|row| (*row).to_string()).collect();

    for (offset, band) in (design.eyes)(mood, tick).into_iter().enumerate() {
        let row = design.eye_top + offset;
        rows[row] = patch(design.body[row], band, design);
    }
    let mouth = design.mouth_row();
    rows[mouth] = patch(design.body[mouth], (design.mouth)(mood), design);

    rows.try_into().expect("row count is fixed")
}

fn patch(row: &str, centre: &str, design: &Design) -> String {
    let mut pixels: Vec<char> = row.chars().collect();
    for (offset, pixel) in centre.chars().enumerate() {
        if pixel != KEEP {
            pixels[design.face_at + offset] = if pixel == IRIS { design.iris } else { pixel };
        }
    }
    pixels.into_iter().collect()
}

#[cfg(test)]
mod tests;
