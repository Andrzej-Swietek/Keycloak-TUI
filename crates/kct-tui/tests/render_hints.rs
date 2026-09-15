//! What the status bar offers on each kind of screen.

use kct_tui::model::{Model, Session};
use kct_tui::view;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn screen(model: &Model) -> String {
    let mut terminal = Terminal::new(TestBackend::new(140, 24)).expect("test terminal");
    terminal
        .draw(|frame| view::render(model, frame))
        .expect("render");

    let buffer = terminal.backend().buffer().clone();
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_read_only_screen_offers_no_write_keys() {
    let on = |id| {
        let resource = kct_core::resources::by_id(id).unwrap();
        screen(&Model::new(
            Session::new("https://kc.test", "avsystem", "admin"),
            resource,
        ))
    };

    let built_in = on("client-profiles-global");
    assert!(!built_in.contains("new"), "{built_in}");
    assert!(!built_in.contains("delete"));

    let editable = on("client-profile-items");
    assert!(editable.contains("new"));
    assert!(editable.contains("delete"));
}
