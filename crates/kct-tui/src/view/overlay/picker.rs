use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, ListState, Paragraph, StatefulWidget};

use crate::model::{Model, Picker};
use crate::view::popup;

pub fn render(model: &Model, picker: &Picker, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;
    let area = popup::centered(area, 60, 65);
    frame.render_widget(Clear, area);

    let block = Block::bordered()
        .border_style(theme.border_style(true))
        .title(format!(" {} ", picker.title))
        .title_bottom(" tab pick · ⏎ confirm · esc cancel ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [query, results] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("› ", theme.title()),
            Span::styled(picker.input.value().to_owned(), theme.base()),
            Span::styled("▏", theme.title()),
            Span::styled(
                match picker.chosen.len() {
                    0 => String::new(),
                    count => format!("   {count} picked"),
                },
                theme.dim(),
            ),
        ])),
        query,
    );

    if picker.loading {
        frame.render_widget(Paragraph::new(" loading… ").style(theme.dim()), results);
        return;
    }

    let items: Vec<ListItem> = picker
        .matches
        .iter()
        .filter_map(|index| picker.options.get(*index).map(|option| (index, option)))
        .map(|(index, option)| {
            let ticked = picker.chosen.contains(index);
            ListItem::new(Line::from(vec![
                Span::styled(if ticked { "[x] " } else { "[ ] " }, theme.title()),
                Span::styled(picker.label_of(option), theme.base()),
            ]))
        })
        .collect();

    let mut state = ListState::default().with_selected(Some(picker.cursor));
    StatefulWidget::render(
        List::new(items).highlight_style(theme.selected()),
        results,
        frame.buffer_mut(),
        &mut state,
    );
}
