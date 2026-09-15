use kct_core::catalog::Tab;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Tabs, Wrap};

use crate::model::{Detail, Model};

use super::form::form;

/// The detail pane: tab strip plus either a form, a nested listing, or raw JSON.
pub fn render(model: &Model, detail: &Detail, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;
    let resource = model.resource();

    let block = Block::bordered()
        .border_style(theme.border_style(true))
        .title(Line::from(vec![
            Span::styled(format!(" {} ", detail.title), theme.title()),
            Span::styled(
                if detail.is_dirty() { "· unsaved " } else { "" },
                theme.danger(),
            ),
        ]))
        .title_bottom(hints(detail));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [strip, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);

    frame.render_widget(
        Tabs::new(resource.tabs.iter().map(Tab::title).collect::<Vec<_>>())
            .select(detail.tab)
            .style(theme.dim())
            .highlight_style(theme.title()),
        strip,
    );

    if detail.loading {
        frame.render_widget(Paragraph::new(" loading… ").style(theme.dim()), body);
        return;
    }
    if let Some(error) = &detail.error {
        frame.render_widget(
            Paragraph::new(error.to_string())
                .style(theme.danger())
                .wrap(Wrap { trim: false }),
            body,
        );
        return;
    }

    match resource.tabs.get(detail.tab) {
        Some(tab @ Tab::Form { .. }) => form(model, detail, tab, frame, body),
        Some(Tab::Nested {
            resource: child, ..
        }) => {
            let child = kct_core::resources::by_id(child).map_or("", |child| child.plural);
            frame.render_widget(
                Paragraph::new(format!("\n  Enter — open {child}")).style(theme.dim()),
                body,
            );
        }
        _ => raw(model, detail, frame, body),
    }
}

/// Draw the raw tab: one `pointer = value` row per JSON leaf.
fn raw(model: &Model, detail: &Detail, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;
    let leaves = crate::update::raw_leaves(detail);
    let width = leaves
        .iter()
        .map(|(pointer, _)| pointer.len())
        .max()
        .unwrap_or(0)
        .min(48);

    let lines: Vec<Line> = leaves
        .iter()
        .enumerate()
        .map(|(index, (pointer, value))| {
            let selected = index == detail.cursor;
            let editing = detail
                .editing
                .as_ref()
                .filter(|edit| edit.field == index)
                .map(|edit| format!("{}▏", edit.input.value()));

            Line::from(vec![
                Span::styled(if selected { " ▸ " } else { "   " }, theme.title()),
                Span::styled(
                    format!("{pointer:width$}  "),
                    if selected { theme.title() } else { theme.dim() },
                ),
                Span::styled(
                    editing
                        .clone()
                        .unwrap_or_else(|| kct_core::json::to_line(value).into_owned()),
                    if editing.is_some() {
                        theme.selected()
                    } else {
                        theme.base()
                    },
                ),
            ])
        })
        .collect();

    let visible = usize::from(area.height).saturating_sub(1).max(1);
    let scroll = detail.cursor.saturating_sub(visible.saturating_sub(1)) as u16;

    frame.render_widget(Paragraph::new(lines).scroll((scroll, 0)), area);
}

fn hints(detail: &Detail) -> Line<'static> {
    let keys = if detail.editing.is_some() {
        " ⏎ commit · esc cancel "
    } else {
        " h/l tabs · j/k move · ⏎ edit · a actions · w save · y copy · esc back "
    };
    Line::from(Span::raw(keys))
}
