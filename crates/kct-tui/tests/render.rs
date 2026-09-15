//! Rendering tests.
//!
//! The view is a pure function of the model, so it can be rendered into an in-memory buffer
//! and asserted on — no terminal, no server, and a regression shows up as a diff in text.

use std::sync::Arc;

use kct_core::capability::ServerProfile;
use kct_tui::model::{Confirm, List, Model, Overlay, Session, Spotlight, UnmanagedAttributes};
use kct_tui::{spotlight, view};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use serde_json::json;

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
                .trim_end()
                .to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn profile() -> ServerProfile {
    ServerProfile::from_server_info(json!({
        "systemInfo": {"version": "26.7.3"},
        "features": [{"name": "organization", "type": "DEFAULT", "enabled": true}]
    }))
}

fn session() -> Session {
    Session {
        profile: profile(),
        ..Session::new("https://kc.example.com", "avsystem", "admin via admin-cli")
    }
}

fn users_with(rows: Vec<serde_json::Value>) -> Model {
    let users = kct_core::resources::by_id("users").unwrap();
    let model = Model::new(session(), users);
    Model {
        list: List {
            rows: Arc::from(rows),
            ..List::default()
        }
        .refiltered(users),
        ..model
    }
}

#[test]
fn a_listing_shows_the_catalogs_columns_and_the_servers_data() {
    let rendered = screen(&users_with(vec![
        json!({"id": "1", "username": "alice", "email": "alice@example.com", "enabled": true}),
        json!({"id": "2", "username": "bob", "enabled": false}),
    ]));

    assert!(rendered.contains("kct avsystem"), "{rendered}");
    assert!(rendered.contains("kc 26.7.3"));
    assert!(rendered.contains("Username"));
    assert!(rendered.contains("alice@example.com"));
    // A disabled account must be obvious at a glance, not a blank cell.
    assert!(rendered.contains("DISABLED"));
    assert!(rendered.contains("2 users"));
}

#[test]
fn an_unsafe_session_says_so_on_every_frame() {
    let model = Model {
        session: Session {
            dry_run: true,
            insecure: true,
            cleartext: Some(Arc::from("http")),
            ..session()
        },
        ..users_with(Vec::new())
    };

    let rendered = screen(&model);
    assert!(rendered.contains("DRY-RUN"));
    assert!(rendered.contains("TLS UNVERIFIED"));
    assert!(rendered.contains("CLEARTEXT"));
}

#[test]
fn a_forbidden_screen_renders_the_remediation_not_just_the_status() {
    let model = Model {
        list: List {
            error: Some(kct_core::Error::Api {
                status: 403,
                method: kct_core::Method::Get,
                path: "/admin/realms/avsystem/clients".into(),
                message: "Forbidden".into(),
                detail: None,
            }),
            ..List::default()
        },
        ..users_with(Vec::new())
    };

    let rendered = screen(&model);
    assert!(rendered.contains("403"));
    assert!(rendered.contains("view-clients"), "{rendered}");
}

#[test]
fn the_palette_lists_matches_and_explains_what_is_unavailable() {
    let old = ServerProfile::from_discovery(kct_core::Version::new(25, 0, 6));
    let entries = spotlight::catalog(&old, &[], None);

    let model = Model {
        session: Session {
            profile: old,
            ..session()
        },
        overlay: Some(Overlay::Spotlight(
            Spotlight::new(entries).researched_with("organiz"),
        )),
        ..users_with(Vec::new())
    };

    let rendered = screen(&model);
    assert!(rendered.contains("commands"));
    assert!(rendered.contains("Organizations"));
    assert!(
        rendered.contains("26.0.0"),
        "the reason must be visible: {rendered}"
    );
}

#[test]
fn a_destructive_confirmation_shows_the_name_that_must_be_typed() {
    let model = Model {
        overlay: Some(Overlay::Confirm(Confirm {
            title: Arc::from("Delete realm"),
            body: Arc::from("Delete realm 'production'? This cannot be undone."),
            danger: kct_core::catalog::Danger::Destructive,
            expect: Some(Arc::from("production")),
            typed: kct_tui::TextInput::default(),
            on_confirm: Arc::new(Vec::new()),
        })),
        ..users_with(Vec::new())
    };

    let rendered = screen(&model);
    assert!(rendered.contains("cannot be undone"));
    assert!(
        rendered.contains("type “production” to proceed"),
        "{rendered}"
    );
}

#[test]
fn a_save_warns_before_keycloak_would_discard_an_attribute() {
    // The text comes from the same check the update path runs; rendering proves it reaches
    // the operator's eyes rather than only existing in a struct.
    let model = Model {
        session: Session {
            unmanaged_attributes: UnmanagedAttributes::Discarded,
            ..session()
        },
        overlay: Some(Overlay::Confirm(Confirm {
            title: Arc::from("Save changes"),
            body: Arc::from(
                "⚠ This realm's user profile declares no policy for unmanaged attributes, so \
                 Keycloak will silently discard 'department'.\n\n{}",
            ),
            danger: kct_core::catalog::Danger::Confirm,
            expect: None,
            typed: kct_tui::TextInput::default(),
            on_confirm: Arc::new(Vec::new()),
        })),
        ..users_with(Vec::new())
    };

    let rendered = screen(&model);
    assert!(rendered.contains("silently discard"), "{rendered}");
}

#[test]
fn the_narrowest_supported_terminal_still_renders() {
    let mut terminal = Terminal::new(TestBackend::new(80, 12)).expect("terminal");
    let model = users_with(vec![json!({"id": "1", "username": "alice"})]);

    terminal
        .draw(|frame| view::render(&model, frame))
        .expect("an 80x12 terminal must not panic");
}
