use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::model::Model;

/// Top line: who we are, where we are, and anything unsafe about the session.
pub fn render(model: &Model, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;

    let version = model.session.profile.version_summary();

    let left = Line::from(vec![
        Span::styled(" kct ", theme.title()),
        Span::styled(model.session.realm.to_string(), theme.base()),
        Span::styled("  ", theme.dim()),
        Span::styled(format!("kc {version}"), theme.dim()),
        Span::styled("  ", theme.dim()),
        Span::styled(model.session.base_url.to_string(), theme.dim()),
    ]);

    let warnings = model.session.standing_warnings();
    let right = Line::from(
        warnings
            .iter()
            .map(|warning| Span::styled(format!(" {warning} "), theme.danger()))
            .chain(std::iter::once(Span::styled(
                format!(" {} ", model.session.account),
                theme.dim(),
            )))
            .collect::<Vec<_>>(),
    );

    frame.render_widget(Paragraph::new(left), area);
    frame.render_widget(Paragraph::new(right).alignment(Alignment::Right), area);
}
