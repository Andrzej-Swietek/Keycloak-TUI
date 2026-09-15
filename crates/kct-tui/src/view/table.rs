use kct_core::catalog::{Resource, Width};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Row, StatefulWidget, Table, TableState};

use crate::model::Model;
use crate::theme::Theme;

/// The listing pane.
pub fn render(model: &Model, frame: &mut Frame, area: Rect) {
    let resource = model.resource();
    let theme = &model.theme;

    let block = Block::bordered()
        .border_style(theme.border_style(!model.nav.sidebar_focused))
        .title(title(model))
        .title_bottom(footer(model));

    if let Some(error) = &model.list.error {
        let body = match error.hint() {
            Some(hint) => format!("{error}\n\n{hint}"),
            None => error.to_string(),
        };
        frame.render_widget(
            ratatui::widgets::Paragraph::new(body)
                .style(theme.danger())
                .wrap(ratatui::widgets::Wrap { trim: false })
                .block(block),
            area,
        );
        return;
    }

    let header = Row::new(
        resource
            .columns
            .iter()
            .map(|column| Cell::from(Span::styled(column.header, theme.title()))),
    );

    let rows: Vec<Row> = model
        .list
        .visible
        .iter()
        .filter_map(|index| model.list.rows.get(*index))
        .map(|row| cells(resource, row, theme))
        .collect();

    let mut state = TableState::default().with_selected(Some(model.list.cursor));
    state = state.with_offset(model.list.scroll);

    StatefulWidget::render(
        Table::new(rows, widths(resource))
            .header(header)
            .block(block)
            .row_highlight_style(theme.selected())
            .highlight_symbol("▌"),
        area,
        frame.buffer_mut(),
        &mut state,
    );
}

fn cells<'a>(resource: &'static Resource, row: &serde_json::Value, theme: &Theme) -> Row<'a> {
    Row::new(
        resource
            .columns
            .iter()
            .map(|column| {
                let value = column.value.render(row);
                let style = theme.cell(column.tone, &value);
                Cell::from(Span::styled(value, style))
            })
            .collect::<Vec<_>>(),
    )
}

fn widths(resource: &'static Resource) -> Vec<Constraint> {
    resource
        .columns
        .iter()
        .map(|column| match column.width {
            Width::Fixed(cells) => Constraint::Length(cells),
            Width::Flex(weight) => Constraint::Fill(weight),
        })
        .collect()
}

fn title(model: &Model) -> Line<'static> {
    let resource = model.resource();
    let crumbs = model.nav.crumbs().join(" › ");
    let shown = model.list.len();
    let counted = match (model.list.total, shown) {
        (Some(total), _) => format!("{shown} of {total}"),
        (None, 1) => format!("1 {}", resource.singular.to_lowercase()),
        (None, count) => format!("{count} {}", resource.plural.to_lowercase()),
    };

    Line::from(vec![
        Span::styled(format!(" {crumbs} "), model.theme.title()),
        Span::styled(format!("{counted} "), model.theme.dim()),
    ])
}

fn footer(model: &Model) -> Line<'static> {
    let theme = &model.theme;

    if model.list.loading {
        return Line::from(Span::styled(" loading… ", theme.dim()));
    }
    if model.list.filtering || !model.list.filter.is_empty() {
        return Line::from(vec![
            Span::styled(" filter: ", theme.dim()),
            Span::styled(model.list.filter.value().to_owned(), theme.base()),
            Span::styled(if model.list.filtering { "▏" } else { " " }, theme.base()),
        ]);
    }

    Line::from(Span::styled(
        format!(" {} ", model.list.query.describe()),
        theme.dim(),
    ))
}
