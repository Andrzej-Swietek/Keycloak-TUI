use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use kct_core::catalog::Ops;

use crate::model::Model;
use crate::msg::Level;

/// Bottom line: the newest notice, or the key hints when there is nothing to say.
pub fn render(model: &Model, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;

    let line = match model.toasts.last() {
        Some(toast) => {
            let style = match toast.notice.level {
                Level::Info => theme.dim(),
                Level::Success => theme.cell(kct_core::catalog::Tone::Good, ""),
                Level::Warning => theme.cell(kct_core::catalog::Tone::Warn, ""),
                Level::Failure => theme.danger(),
            };
            // Multi-line notices carry a remediation hint; the status bar shows the first
            // line and the rest stays available in the request log.
            let text = toast.notice.text.replace('\n', " · ");
            Line::from(Span::styled(format!(" {text}"), style))
        }
        None => Line::from(hints(model)),
    };

    frame.render_widget(Paragraph::new(line), area);
}

/// Key hints for the screen on show, omitting the writes it does not accept.
fn hints<'a>(model: &Model) -> Vec<Span<'a>> {
    let theme = &model.theme;
    let ops = model.resource().ops;
    let hint = |key: &str, what: &str| {
        [
            Span::styled(format!("{key} "), theme.title()),
            Span::styled(format!("{what}  "), theme.dim()),
        ]
    };

    std::iter::once(Span::styled(" : ", theme.title()))
        .chain(Some(Span::styled("commands  ", theme.dim())))
        .chain(hint("/", "filter"))
        .chain(hint("⏎", "open"))
        .chain(ops.has(Ops::CREATE).then(|| hint("n", "new")).into_iter().flatten())
        .chain(ops.has(Ops::DELETE).then(|| hint("x", "delete")).into_iter().flatten())
        .chain(hint("a", "actions"))
        .chain(hint("?", "help"))
        .collect()
}
