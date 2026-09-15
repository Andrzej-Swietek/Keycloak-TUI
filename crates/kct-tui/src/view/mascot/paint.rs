use ratatui::style::Color;
use ratatui::text::{Line, Span};

use super::custom::Art;
use super::{Character, Mood, palette, sprite};

const UPPER_HALF: &str = "\u{2580}";
const LOWER_HALF: &str = "\u{2584}";

/// A sprite ready to draw, with the size the panel has to reserve for it.
pub struct Portrait<'a> {
    pub lines: Vec<Line<'a>>,
    pub cols: u16,
    pub rows: u16,
}

/// One of the compiled-in designs, patched with the mood's expression.
pub fn builtin<'a>(character: Character, mood: Mood, tick: u64) -> Portrait<'a> {
    let top = character.design().top();
    let frame = sprite::frame(character, mood, tick);
    let lines: Vec<Line> = frame[top..]
        .chunks(2)
        .map(|pair| {
            paint(
                pair[0].chars().map(palette::ink),
                pair[1].chars().map(palette::ink),
            )
        })
        .collect();

    Portrait {
        cols: u16::try_from(sprite::WIDTH).unwrap_or(u16::MAX),
        rows: u16::try_from(lines.len()).unwrap_or(0),
        lines,
    }
}

/// Paint loaded art. An odd-height image gets a blank final pixel row; no drawn row is
/// dropped.
pub fn custom<'a>(art: &Art) -> Portrait<'a> {
    let top = art.top();
    let lines: Vec<Line> = (top..art.height())
        .step_by(2)
        .map(|row| {
            paint(
                art.row(row).iter().copied(),
                art.row(row + 1).iter().copied().chain(std::iter::repeat(None)),
            )
        })
        .collect();

    Portrait {
        cols: u16::try_from(art.width).unwrap_or(u16::MAX),
        rows: u16::try_from(lines.len()).unwrap_or(0),
        lines,
    }
}

/// Two pixel rows become one text row of half-blocks.
///
/// A half-block painted with a `Reset` colour shows the terminal's own foreground, which is a
/// pale rectangle around the sprite. So a cell with one transparent pixel picks the glyph that
/// puts the opaque pixel in the ink and leaves the rest genuinely unpainted.
fn paint<'a>(
    upper: impl Iterator<Item = Option<Color>>,
    lower: impl Iterator<Item = Option<Color>>,
) -> Line<'a> {
    Line::from(
        upper
            .zip(lower)
            .map(|(top, bottom)| cell(top, bottom))
            .collect::<Vec<_>>(),
    )
}

fn cell<'a>(top: Option<Color>, bottom: Option<Color>) -> Span<'a> {
    let ink = |colour| ratatui::style::Style::default().fg(colour);
    match (top, bottom) {
        (None, None) => Span::raw(" "),
        (None, Some(colour)) => Span::styled(LOWER_HALF, ink(colour)),
        (Some(colour), None) => Span::styled(UPPER_HALF, ink(colour)),
        (Some(above), Some(below)) => Span::styled(UPPER_HALF, ink(above).bg(below)),
    }
}
