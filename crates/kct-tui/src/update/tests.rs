use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use kct_core::catalog::Resource;
use serde_json::{Value, json};

use crate::effect::Effect;
use crate::model::{Model, Overlay, Session};
use crate::msg::{Fetch, Msg};

use super::{Transition, update};

/// A page of rows as the loader would deliver it. Most tests do not care what document the
/// rows came out of; the ones that do build the envelope themselves.
fn page(rows: Vec<serde_json::Value>) -> crate::msg::Page {
    crate::msg::Page {
        envelope: std::sync::Arc::new(serde_json::Value::Array(rows.clone())),
        rows: std::sync::Arc::from(rows),
    }
}


fn users() -> &'static Resource {
    kct_core::resources::by_id("users").unwrap()
}

fn model() -> Model {
    Model::new(
        Session::new("https://kc.test", "avsystem", "admin"),
        users(),
    )
}

fn key(code: KeyCode) -> Msg {
    Msg::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn typed(model: Model, text: &str) -> Model {
    text.chars().fold(model, |model, character| {
        update(model, key(KeyCode::Char(character))).model
    })
}

fn loaded(model: Model, rows: Vec<Value>) -> Model {
    let token = model.list.token;
    update(
        model,
        Msg::Rows {
            token,
            result: Ok(page(rows)),
        },
    )
    .model
}

fn listing(rows: Vec<Value>) -> Model {
    let started = super::reload(model());
    loaded(started.model, rows)
}

#[test]
fn opening_a_screen_asks_for_its_rows() {
    let Transition { model, effects } = super::reload(model());

    assert!(model.list.loading);
    match effects.as_slice() {
        [
            Effect::Load {
                fetch, path, token, ..
            },
        ] => {
            assert_eq!(*fetch, Fetch::Rows);
            assert_eq!(path.as_str(), "/admin/realms/avsystem/users");
            assert_eq!(*token, model.list.token);
        }
        other => panic!("expected one load effect, got {other:?}"),
    }
}

#[test]
fn a_reply_for_an_abandoned_request_is_ignored() {
    let model = listing(vec![json!({"id": "1", "username": "alice"})]);

    // A slower, older request answering after the operator moved on must not clobber the view.
    let stale = update(
        model,
        Msg::Rows {
            token: 999,
            result: Ok(page(vec![json!({"id": "9", "username": "ghost"})])),
        },
    )
    .model;

    assert_eq!(stale.list.len(), 1);
    assert_eq!(stale.list.selected().unwrap()["username"], json!("alice"));
}

#[test]
fn filtering_narrows_the_visible_rows_without_refetching() {
    let model = listing(vec![
        json!({"id": "1", "username": "alice"}),
        json!({"id": "2", "username": "bob"}),
    ]);

    let filtering = update(model, key(KeyCode::Char('/'))).model;
    assert!(filtering.list.filtering);

    let filtered = typed(filtering, "ali");
    assert_eq!(filtered.list.len(), 1);
    assert_eq!(
        filtered.list.rows.len(),
        2,
        "rows are kept, only the view narrows"
    );
}

#[test]
fn deleting_requires_typing_the_name_and_only_then_sends_the_request() {
    let model = listing(vec![json!({"id": "u1", "username": "alice"})]);

    let confirming = update(model, key(KeyCode::Char('x'))).model;
    let Some(Overlay::Confirm(confirm)) = &confirming.overlay else {
        panic!("expected a confirmation");
    };
    assert_eq!(confirm.expect.as_deref(), Some("alice"));

    // Enter with the wrong text does nothing.
    let still_open = update(confirming, key(KeyCode::Enter));
    assert!(still_open.effects.is_empty());
    assert!(still_open.model.overlay.is_some());

    let typed_name = typed(still_open.model, "alice");
    let Transition { model, effects } = update(typed_name, key(KeyCode::Enter));

    assert!(model.overlay.is_none());
    match effects.as_slice() {
        [Effect::Call { method, path, .. }] => {
            assert_eq!(*method, kct_core::Method::Delete);
            assert_eq!(path.as_str(), "/admin/realms/avsystem/users/u1");
        }
        other => panic!("expected one delete, got {other:?}"),
    }
}

#[test]
fn dry_run_refuses_writes_before_they_reach_the_network() {
    let model = Model {
        session: Session {
            dry_run: true,
            ..Session::new("https://kc.test", "avsystem", "admin")
        },
        ..listing(vec![json!({"id": "u1", "username": "alice"})])
    };

    let Transition { model, effects } = update(model, key(KeyCode::Char('x')));

    assert!(effects.is_empty());
    assert!(
        model.overlay.is_none(),
        "not even a confirmation is offered"
    );
    assert!(
        model
            .toasts
            .iter()
            .any(|toast| toast.notice.text.contains("dry-run"))
    );
}

#[test]
fn a_screen_the_server_cannot_serve_explains_itself_instead_of_navigating() {
    let old = Model {
        session: Session {
            profile: kct_core::capability::ServerProfile::from_discovery(kct_core::Version::new(
                25, 0, 6,
            )),
            ..Session::new("https://kc.test", "avsystem", "admin")
        },
        ..model()
    };
    let organizations = kct_core::resources::by_id("organizations").unwrap();

    let Transition { model, effects } = super::nav::go_to(old, organizations);

    assert!(effects.is_empty());
    assert_eq!(model.resource().id, "users", "navigation did not happen");
    assert!(model.toasts[0].notice.text.contains("26.0.0"));
}
