//! A pixel-art mascot in the bottom-right corner, showing what the session is doing.
//!
//! [`Mood::of`] reads the state; the sprite and the caption beside it both follow from it, so
//! a glance at the corner reports whether a request is in flight, whether the last write
//! landed, and whether a destructive confirmation is open.
//!
//! Each terminal cell carries two pixels as the foreground and background of a half-block, so
//! the sprite has twice the vertical resolution of the text grid. Characters come either from
//! [`sprite`] (compiled in) or from [`custom`] (PNGs the operator supplies); [`render`] draws
//! nothing on a pane too small for the panel plus a usable listing.

pub mod custom;
mod mood;
mod paint;
mod palette;
pub mod preview;
mod sprite;

pub use custom::{CustomMascot, Scan};
pub use mood::Mood;
pub use sprite::Character;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use crate::model::{Mascot, Model};

use paint::Portrait;

const TEXT_COLS: u16 = 14;
const GAP: u16 = 1;

/// Room the panel must leave for the thing an operator actually came for. Measured against
/// the pane it is handed, not the terminal, which is two rows taller.
const SIDEBAR: u16 = 26;
const MIN_LISTING_WIDTH: u16 = 24;
const MIN_ROWS_ABOVE: u16 = 6;

/// The panel is only as big as the sprite drawn in it, so a bust does not reserve the rows a
/// full figure needs and a small drop-in image does not reserve a large one's width.
fn fits(area: Rect, portrait: &Portrait) -> bool {
    area.width >= SIDEBAR + MIN_LISTING_WIDTH + portrait.cols + GAP + TEXT_COLS + 4
        && area.height >= portrait.rows + 2 + MIN_ROWS_ABOVE
}

pub fn render(model: &Model, frame: &mut Frame, area: Rect) {
    let mood = Mood::of(model);
    let Some(drawn) = model
        .prefs
        .mascot
        .and_then(|who| portrait(model, who, mood))
        .filter(|drawn| fits(area, drawn))
    else {
        return;
    };

    let width = drawn.cols + GAP + TEXT_COLS + 2;
    let panel = drawn.rows + 2;
    let corner = Rect {
        x: area.x + area.width - width - 1,
        y: area.y + area.height - panel - 1,
        width,
        height: panel,
    };

    let block = Block::bordered().border_style(model.theme.border_style(false));
    let inner = block.inner(corner);

    frame.render_widget(Clear, corner);
    frame.render_widget(block, corner);

    let [sprite_area, _, caption] = Layout::horizontal([
        Constraint::Length(drawn.cols),
        Constraint::Length(GAP),
        Constraint::Min(0),
    ])
    .areas(inner);

    frame.render_widget(Paragraph::new(drawn.lines), sprite_area);
    frame.render_widget(say(model, mood), caption);
}

fn portrait<'a>(model: &Model, who: Mascot, mood: Mood) -> Option<Portrait<'a>> {
    match who {
        Mascot::Builtin(character) => Some(paint::builtin(character, mood, model.ticks)),
        Mascot::Custom(index) => model
            .mascots
            .found
            .get(index)
            .map(|art| paint::custom(art.frame(mood))),
    }
}

/// What to call whoever is in the corner, built-in or dropped in.
#[must_use]
pub fn name_of(model: &Model, who: Mascot) -> String {
    match who {
        Mascot::Builtin(character) => character.name().to_owned(),
        Mascot::Custom(index) => model
            .mascots
            .found
            .get(index)
            .map_or_else(|| "missing".to_owned(), |art| art.name.to_string()),
    }
}

fn say<'a>(model: &Model, mood: Mood) -> Paragraph<'a> {
    let theme = &model.theme;
    let tint = match mood {
        Mood::Alarmed => theme.danger(),
        Mood::Worried => theme.cell(kct_core::catalog::Tone::Warn, ""),
        Mood::Happy => theme.cell(kct_core::catalog::Tone::Good, ""),
        Mood::Busy | Mood::Sly => theme.dim(),
        Mood::Idle => theme.cell(kct_core::catalog::Tone::Primary, ""),
    };

    Paragraph::new(vec![
        Line::from(Span::styled(
            mood.caption(),
            tint.add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(mood.detail(model), theme.dim())),
    ])
    .wrap(Wrap { trim: true })
}

#[cfg(test)]
mod tests;
