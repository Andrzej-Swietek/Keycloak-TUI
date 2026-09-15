use kct_core::catalog::{Field, Tab};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::model::{Detail, Model};
use crate::update;

use super::suggestions::dropdown;

pub(super) fn form(model: &Model, detail: &Detail, tab: &Tab, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;
    let fields = update::visible_fields(model, tab);
    let label_width = fields
        .iter()
        .map(|field| field.label.len())
        .max()
        .unwrap_or(0)
        .min(30);

    let lines: Vec<Line> = fields
        .iter()
        .enumerate()
        .map(|(index, field)| row(model, detail, index, field, label_width))
        .collect();

    let help = fields
        .get(detail.cursor)
        .map(|field| field.help)
        .unwrap_or_default();

    let [values, hint] = Layout::vertical([Constraint::Min(0), Constraint::Length(2)]).areas(area);

    frame.render_widget(Paragraph::new(lines).scroll((detail.scroll, 0)), values);
    frame.render_widget(
        Paragraph::new(Line::from(help.to_owned()))
            .style(theme.dim())
            .wrap(Wrap { trim: true }),
        hint,
    );
    dropdown(model, detail, frame, values, label_width);
}

fn row<'a>(
    model: &Model,
    detail: &Detail,
    index: usize,
    field: &'static Field,
    label_width: usize,
) -> Line<'a> {
    let theme = &model.theme;
    let selected = index == detail.cursor;

    let editing = detail
        .editing
        .as_ref()
        .filter(|edit| edit.field == index)
        .map(|edit| format!("{}▏", edit.value()));

    let value = editing
        .clone()
        .unwrap_or_else(|| field.display(&detail.draft));

    Line::from(vec![
        Span::styled(if selected { " ▸ " } else { "   " }, theme.title()),
        Span::styled(
            format!("{:label_width$}  ", field.label),
            if selected { theme.title() } else { theme.dim() },
        ),
        Span::styled(
            value,
            if editing.is_some() {
                theme.selected()
            } else {
                theme.base()
            },
        ),
    ])
}
