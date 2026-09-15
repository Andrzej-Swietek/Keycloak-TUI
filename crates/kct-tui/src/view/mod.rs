//! Rendering. Every function here takes `&Model` and writes to a frame; none of them decide
//! anything.

mod detail;
mod form;
mod header;
mod mascot;
mod overlay;
mod popup;
mod sidebar;
mod status;
mod suggestions;
mod table;

pub use mascot::{
    Character, CustomMascot, Scan, custom as mascot_art, name_of as mascot_name,
    preview as mascot_preview,
};

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

use crate::model::Model;

/// Width of the navigation tree. Wide enough for the longest screen name, narrow enough to
/// leave a listing readable on an 80-column terminal.
const SIDEBAR_WIDTH: u16 = 26;

pub fn render(model: &Model, frame: &mut Frame) {
    let [top, middle, bottom] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    header::render(model, frame, top);

    let [tree, main] =
        Layout::horizontal([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(0)]).areas(middle);

    sidebar::render(model, frame, tree);

    match &model.detail {
        Some(open) => detail::render(model, open, frame, main),
        None => table::render(model, frame, main),
    }

    mascot::render(model, frame, middle);
    status::render(model, frame, bottom);

    if let Some(open) = &model.overlay {
        overlay::render(model, open, frame, frame.area());
    }
}

/// Rows the listing can show, given the terminal height. The runtime feeds this back into
/// the model so paging and scrolling match what is actually visible.
#[must_use]
pub fn viewport_rows(height: u16) -> usize {
    // One line each for the header and status bar, two for the table's border, one header row.
    usize::from(height.saturating_sub(5)).max(1)
}
