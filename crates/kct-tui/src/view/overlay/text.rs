use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, ListState, Paragraph, StatefulWidget, Wrap};

use crate::model::Model;
use crate::view::popup;

const HELP: &str = "\
Navigation
  tab          move focus between the tree and the list
  j / k        move          g / G   first / last
  ⏎            open          esc     back one level
  ] / [        next / previous page

Finding things
  :            command line — vim commands (:w :q :q! :wq :e) and fuzzy palette search
  /            filter the rows already fetched
  s            search on the server (users and groups also accept attributes)

Changing things
  n            create        x       delete
  ⏎            edit the highlighted field (a checkbox toggles in place)
  w  or  :w    save — shows the exact change set before sending
  :q  :q!      close the view; :q refuses while there are unsaved changes
  y            copy the selected object as JSON

Tokens and files
  : inspect    decode your own token: scope, audience, roles, expiry
  : get token   obtain one as the selected client and see what it really receives
  : export      open the realm representation; s writes it to a file
  : import      read a realm export back (existing objects are skipped)

Session
  L            request log — every admin API call this session made
  ?            this help     q / ctrl-c   quit

Every object has a JSON tab. If a Keycloak version adds a field this build does not know
about, that tab still shows it, still edits it, and a save never drops it.";

pub fn help(model: &Model, scroll: u16, frame: &mut Frame, area: Rect) {
    let area = popup::centered(area, 72, 80);
    frame.render_widget(Clear, area);

    frame.render_widget(
        Paragraph::new(HELP)
            .style(model.theme.base())
            .scroll((scroll, 0))
            .block(
                Block::bordered()
                    .border_style(model.theme.border_style(true))
                    .title(" help ")
                    .title_bottom(" j/k scroll · esc close "),
            ),
        area,
    );
}

pub fn viewer(model: &Model, title: &str, body: &str, scroll: u16, frame: &mut Frame, area: Rect) {
    let area = popup::centered(area, 80, 80);
    frame.render_widget(Clear, area);

    frame.render_widget(
        Paragraph::new(body.to_owned())
            .style(model.theme.base())
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0))
            .block(
                Block::bordered()
                    .border_style(model.theme.border_style(true))
                    .title(format!(" {title} "))
                    .title_bottom(" j/k scroll · y copy · s save to file · esc close "),
            ),
        area,
    );
}

pub fn request_log(model: &Model, cursor: usize, frame: &mut Frame, area: Rect) {
    let theme = &model.theme;
    let area = popup::centered(area, 80, 70);
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = model
        .log
        .iter()
        .rev()
        .map(|line| {
            let (status, style) = match &line.outcome {
                Ok(code) => (code.to_string(), theme.dim()),
                Err(reason) => (reason.clone(), theme.danger()),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<6} ", line.method.as_str()), theme.title()),
                Span::styled(format!("{status:<5} "), style),
                Span::styled(line.path.clone(), theme.base()),
            ]))
        })
        .collect();

    let mut state = ListState::default().with_selected(Some(cursor));
    StatefulWidget::render(
        List::new(items).highlight_style(theme.selected()).block(
            Block::bordered()
                .border_style(theme.border_style(true))
                .title(" request log ")
                .title_bottom(" newest first · esc close "),
        ),
        area,
        frame.buffer_mut(),
        &mut state,
    );
}
