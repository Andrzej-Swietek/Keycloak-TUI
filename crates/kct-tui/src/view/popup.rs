use ratatui::layout::{Constraint, Flex, Layout, Rect};

/// A centred rectangle of `width_percent` by `height_percent` of `area`, at least 20 columns
/// by 3 rows where `area` allows and never larger than `area`.
#[must_use]
pub fn centered(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let width = (area.width * width_percent / 100).clamp(20.min(area.width), area.width);
    let height = (area.height * height_percent / 100).clamp(3.min(area.height), area.height);

    let [row] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [cell] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(row);

    cell
}
