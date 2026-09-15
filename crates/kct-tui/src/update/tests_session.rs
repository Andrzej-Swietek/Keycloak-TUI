use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use kct_core::catalog::Resource;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::effect::Effect;
use crate::model::{Model, Overlay, Session};
use crate::msg::Msg;

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
fn the_palette_runs_an_action_belonging_to_another_screen() {
    let model = listing(vec![json!({"id": "u1", "username": "alice"})]);
    let open = update(model, key(KeyCode::Char(':'))).model;
    assert!(matches!(open.overlay, Some(Overlay::Spotlight(_))));

    let searched = typed(open, "create group");
    let Transition { model, .. } = update(searched, key(KeyCode::Enter));

    assert_eq!(model.resource().id, "groups");
    assert!(matches!(model.overlay, Some(Overlay::Prompt(_))));
}

#[test]
fn escape_climbs_out_of_a_nested_view_one_level_at_a_time() {
    let model = listing(vec![json!({"id": "u1", "username": "alice"})]);
    let opened = super::nav::open_selected(model).model;
    assert!(opened.detail.is_some());

    let closed = update(opened, key(KeyCode::Esc)).model;
    assert!(closed.detail.is_none());
    assert_eq!(closed.resource().id, "users");
}

#[test]
fn an_empty_list_never_panics_on_navigation_keys() {
    let empty = listing(Vec::new());
    let after = [
        KeyCode::Char('j'),
        KeyCode::Char('k'),
        KeyCode::Char('G'),
        KeyCode::Enter,
        KeyCode::Char('x'),
        KeyCode::Char('y'),
    ]
    .into_iter()
    .fold(empty, |model, code| update(model, key(code)).model);

    assert!(after.list.is_empty());
    assert!(!after.quit);
}

#[test]
fn list_state_survives_a_realm_switch_by_being_rebuilt() {
    let model = listing(vec![json!({"id": "u1", "username": "alice"})]);
    let Transition { model, effects } = super::nav::switch_realm(model, "master".to_owned());

    assert_eq!(&*model.session.realm, "master");
    assert_eq!(model.resource().id, "users");
    match effects.as_slice() {
        [Effect::Load { path, .. }] => assert_eq!(path.as_str(), "/admin/realms/master/users"),
        other => panic!("expected a reload, got {other:?}"),
    }
}

#[test]
fn ctrl_c_quits_from_anywhere_including_a_modal() {
    let model = update(listing(Vec::new()), key(KeyCode::Char('?'))).model;
    assert!(model.overlay.is_some());

    let Transition { model, effects } = update(
        model,
        Msg::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
    );

    assert!(model.quit);
    assert!(matches!(effects.as_slice(), [Effect::Quit]));
}

#[test]
fn a_failed_write_reports_the_servers_own_message() {
    let model = listing(Vec::new());
    let error = kct_core::Error::Api {
        status: 409,
        method: kct_core::Method::Post,
        path: "/admin/realms/avsystem/users".into(),
        message: "User exists with same username".into(),
        detail: None,
    };

    let after = update(
        model,
        Msg::Done {
            title: "create user".to_owned(),
            result: Err(error),
        },
    )
    .model;

    let toast = &after.toasts[0].notice;
    assert!(toast.text.contains("User exists with same username"));
    assert!(
        toast.text.contains("already exists"),
        "the hint is shown too"
    );
}

#[test]
fn arc_backed_state_keeps_cloning_the_model_cheap() {
    let model = listing((0..5_000).map(|i| json!({"id": i.to_string()})).collect());
    let clone = model.clone();

    assert!(Arc::ptr_eq(&model.list.rows, &clone.list.rows));
}
