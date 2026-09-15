use kct_core::capability::Support;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, StatefulWidget};

use crate::model::{Model, SidebarRow, sidebar_rows};

/// Draw the navigation tree, greying the screens [`sidebar_rows`] marks unsupported.
pub fn render(model: &Model, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;
    let current = model.resource().id;

    let items: Vec<ListItem> = sidebar_rows()
        .into_iter()
        .map(|row| match row {
            SidebarRow::Heading(section) => ListItem::new(Line::from(Span::styled(
                format!(" {}", section.title().to_uppercase()),
                theme.title(),
            ))),
            SidebarRow::Item(resource) => {
                let available = model.session.profile.evaluate(&resource.requirement);
                let style = match (&available, resource.id == current) {
                    (Support::Unavailable(_), _) => theme.dim(),
                    (_, true) => theme.cell(kct_core::catalog::Tone::Primary, ""),
                    _ => theme.base(),
                };
                let marker = if resource.id == current { "▸" } else { " " };
                ListItem::new(Line::from(Span::styled(
                    format!("  {marker} {}", resource.plural),
                    style,
                )))
            }
        })
        .collect();

    let block = Block::bordered()
        .border_style(theme.border_style(model.nav.sidebar_focused))
        .title(" navigate ");

    let mut state = ListState::default();
    if model.nav.sidebar_focused {
        state.select(Some(model.nav.sidebar_cursor));
    }

    StatefulWidget::render(
        List::new(items)
            .block(block)
            .highlight_style(theme.selected()),
        area,
        frame.buffer_mut(),
        &mut state,
    );
}
