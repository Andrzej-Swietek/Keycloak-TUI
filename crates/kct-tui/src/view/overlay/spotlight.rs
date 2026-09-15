use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, ListState, Paragraph, StatefulWidget};

use crate::model::{Model, Spotlight};
use crate::view::popup;

pub fn render(model: &Model, state: &Spotlight, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;
    let area = popup::centered(area, 70, 70);
    frame.render_widget(Clear, area);

    let block = Block::bordered()
        .border_style(theme.border_style(true))
        .title(" commands ")
        .title_bottom(Line::from(Span::raw(format!(
            " {} of {} ",
            state.matches.len(),
            state.entries.len()
        ))));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [query, results] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(": ", theme.title()),
            Span::styled(state.input.value().to_owned(), theme.base()),
            Span::styled("▏", theme.title()),
            Span::styled(
                state
                    .ex
                    .map(|ex| format!("   ⏎ {}", ex.describe()))
                    .unwrap_or_default(),
                theme.cell(kct_core::catalog::Tone::Good, ""),
            ),
        ])),
        query,
    );

    let items: Vec<ListItem> = state
        .matches
        .iter()
        .filter_map(|index| state.entries.get(*index))
        .map(|entry| {
            let title = if entry.available {
                Span::styled(entry.title.clone(), theme.base())
            } else {
                Span::styled(entry.title.clone(), theme.dim())
            };
            let note = entry.note.clone().unwrap_or_else(|| entry.subtitle.clone());

            ListItem::new(vec![
                Line::from(title),
                Line::from(Span::styled(format!("   {note}"), theme.dim())),
            ])
        })
        .collect();

    let mut list_state = ListState::default().with_selected(Some(state.cursor));
    StatefulWidget::render(
        List::new(items).highlight_style(theme.selected()),
        results,
        frame.buffer_mut(),
        &mut list_state,
    );
}
