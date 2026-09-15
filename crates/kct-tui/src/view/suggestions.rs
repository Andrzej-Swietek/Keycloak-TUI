use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::model::{Detail, Model};

const MAX_ROWS: usize = 8;
const MAX_WIDTH: u16 = 44;
const MIN_WIDTH: u16 = 20;

/// Shown along the frame's foot; the frame is at least [`MIN_WIDTH`] to fit it.
const KEYS: &str = " ⇥ pick · ⏎ take ";

/// Draw the values the field being edited accepts, as a list under that field.
///
/// Draws nothing when no field is being edited, when nothing matches what has been typed, or
/// when `values` is under three rows tall.
pub(super) fn dropdown(
    model: &Model,
    detail: &Detail,
    frame: &mut Frame,
    values: Rect,
    label_width: usize,
) {
    let Some(edit) = detail.editing.as_ref() else {
        return;
    };
    if edit.matches.is_empty() || values.height < 3 {
        return;
    }

    let rows = edit.matches.len().min(MAX_ROWS);
    let area = place(detail, values, label_width, widest(edit), rows);
    let start = match edit.highlight {
        Some(highlight) if highlight >= rows => highlight + 1 - rows,
        _ => 0,
    };

    let pad = usize::from(area.width).saturating_sub(2);
    let lines: Vec<Line> = edit
        .matches
        .iter()
        .skip(start)
        .take(rows)
        .enumerate()
        .filter_map(|(offset, index)| {
            let value = edit.suggestions.get(*index)?;
            let picked = edit.highlight == Some(start + offset);
            Some(Line::from(Span::styled(
                format!("{value:<pad$}"),
                if picked {
                    model.theme.selected()
                } else {
                    model.theme.base()
                },
            )))
        })
        .collect();

    let block = Block::bordered()
        .border_style(model.theme.border_style(true))
        .title(count(edit))
        .title_bottom(KEYS);

    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn widest(edit: &crate::model::FieldEdit) -> u16 {
    let longest = edit
        .matches
        .iter()
        .filter_map(|index| edit.suggestions.get(*index))
        .map(|value| value.chars().count())
        .max()
        .unwrap_or(0);
    u16::try_from(longest).unwrap_or(MAX_WIDTH)
}

/// Under the field, unless the pane runs out underneath — then above it.
fn place(detail: &Detail, values: Rect, label_width: usize, widest: u16, rows: usize) -> Rect {
    let width = widest
        .saturating_add(2)
        .clamp(MIN_WIDTH, MAX_WIDTH)
        .min(values.width);
    let height = u16::try_from(rows).unwrap_or(MAX_ROWS as u16) + 2;

    let field_row = detail
        .editing
        .as_ref()
        .map_or(0, |edit| u16::try_from(edit.field).unwrap_or(0));
    let anchor = values.y + field_row.saturating_sub(detail.scroll);

    let column = values.x + 3 + u16::try_from(label_width).unwrap_or(0) + 2;
    let x = column.min(values.right().saturating_sub(width));
    let y = if anchor + 1 + height <= values.bottom() {
        anchor + 1
    } else {
        anchor.saturating_sub(height).max(values.y)
    };

    Rect {
        x,
        y,
        width,
        height: height.min(values.bottom().saturating_sub(y)),
    }
}

fn count(edit: &crate::model::FieldEdit) -> String {
    if edit.matches.len() == edit.suggestions.len() {
        format!(" {} options ", edit.suggestions.len())
    } else {
        format!(" {} of {} ", edit.matches.len(), edit.suggestions.len())
    }
}
