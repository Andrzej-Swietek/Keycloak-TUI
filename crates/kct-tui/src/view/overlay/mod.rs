mod attributes;
mod dialog;
mod picker;
mod spotlight;
mod text;

use ratatui::Frame;
use ratatui::layout::Rect;

use crate::model::{Model, Overlay};

pub fn render(model: &Model, overlay: &Overlay, frame: &mut Frame, area: Rect) {
    match overlay {
        Overlay::Spotlight(state) => spotlight::render(model, state, frame, area),
        Overlay::Picker(state) => picker::render(model, state, frame, area),
        Overlay::Attributes(state) => attributes::render(model, state, frame, area),
        Overlay::Confirm(state) => dialog::confirm(model, state, frame, area),
        Overlay::Prompt(state) => dialog::prompt(model, state, frame, area),
        Overlay::Help { scroll } => text::help(model, *scroll, frame, area),
        Overlay::Viewer {
            title,
            body,
            scroll,
        } => {
            text::viewer(model, title, body, *scroll, frame, area);
        }
        Overlay::RequestLog { cursor } => text::request_log(model, *cursor, frame, area),
    }
}
