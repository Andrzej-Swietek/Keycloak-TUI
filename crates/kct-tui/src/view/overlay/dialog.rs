use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use crate::model::{Confirm, Model, Prompt};
use crate::view::popup;

pub fn confirm(model: &Model, state: &Confirm, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;
    let area = popup::centered(area, 60, 40);
    frame.render_widget(Clear, area);

    let block = Block::bordered()
        .border_style(theme.danger())
        .title(format!(" {} ", state.title))
        .title_bottom(match state.expect {
            Some(_) => " type the name to confirm · esc cancel ",
            None => " ⏎/y confirm · esc cancel ",
        });
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [body, entry] = Layout::vertical([Constraint::Min(0), Constraint::Length(2)]).areas(inner);

    frame.render_widget(
        Paragraph::new(state.body.to_string())
            .style(theme.base())
            .wrap(Wrap { trim: false }),
        body,
    );

    if let Some(expected) = &state.expect {
        let matches = state.is_satisfied();
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    format!("type “{expected}” to proceed"),
                    theme.dim(),
                )),
                Line::from(vec![
                    Span::styled("› ", theme.title()),
                    Span::styled(
                        state.typed.value().to_owned(),
                        if matches {
                            theme.title()
                        } else {
                            theme.danger()
                        },
                    ),
                    Span::styled("▏", theme.base()),
                ]),
            ]),
            entry,
        );
    }
}

pub fn prompt(model: &Model, state: &Prompt, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;
    let area = popup::centered(area, 60, 25);
    frame.render_widget(Clear, area);

    let block = Block::bordered()
        .border_style(theme.border_style(true))
        .title(format!(" {} ", state.title))
        .title_bottom(" ⏎ confirm · esc cancel ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [help, entry] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(inner);

    frame.render_widget(
        Paragraph::new(state.help.to_string())
            .style(theme.dim())
            .wrap(Wrap { trim: true }),
        help,
    );

    let shown = if state.secret {
        "•".repeat(state.input.value().chars().count())
    } else {
        state.input.value().to_owned()
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("› ", theme.title()),
            Span::styled(shown, theme.base()),
            Span::styled("▏", theme.title()),
        ])),
        entry,
    );
}
