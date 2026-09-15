use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Block, Cell, Clear, Row, StatefulWidget, Table, TableState};

use crate::model::{AttributeColumn, Attributes, Model};
use crate::view::popup;

pub fn render(model: &Model, editor: &Attributes, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;
    let area = popup::centered(area, 70, 60);
    frame.render_widget(Clear, area);

    let block = Block::bordered()
        .border_style(theme.border_style(true))
        .title(format!(" {} ", editor.title))
        .title_bottom(if editor.editing.is_some() {
            " ⏎ commit · esc cancel "
        } else {
            " ⏎ edit · tab key/value · a add · d delete · esc apply "
        });

    let rows: Vec<Row> = editor
        .rows
        .iter()
        .enumerate()
        .map(|(index, (key, value))| cells(editor, index, key, value, theme))
        .collect();

    let mut state = TableState::default().with_selected(Some(editor.cursor));
    StatefulWidget::render(
        Table::new(rows, [Constraint::Fill(2), Constraint::Fill(5)])
            .header(Row::new([
                Cell::from(Span::styled("Key", theme.title())),
                Cell::from(Span::styled("Value", theme.title())),
            ]))
            .block(block)
            .row_highlight_style(theme.selected()),
        area,
        frame.buffer_mut(),
        &mut state,
    );
}

fn cells<'a>(
    editor: &Attributes,
    index: usize,
    key: &str,
    value: &str,
    theme: &crate::theme::Theme,
) -> Row<'a> {
    let live = editor.editing.as_ref().filter(|_| index == editor.cursor);

    let rendered = |column: AttributeColumn, text: &str| match live {
        Some(input) if editor.column == column => {
            Span::styled(format!("{}▏", input.value()), theme.selected())
        }
        _ if editor.column == column && index == editor.cursor => {
            Span::styled(text.to_owned(), theme.title())
        }
        _ => Span::styled(text.to_owned(), theme.base()),
    };

    // A key the realm's user profile does not declare is flagged: a save would drop it.
    let key_span = if editor.undeclared().contains(&key) {
        Span::styled(
            format!("{key} ⚠"),
            theme.cell(kct_core::catalog::Tone::Warn, ""),
        )
    } else {
        rendered(AttributeColumn::Key, key)
    };

    Row::new(vec![
        Cell::from(key_span),
        Cell::from(rendered(AttributeColumn::Value, value)),
    ])
}
