use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use kct_core::catalog::Resource;
use serde_json::{Value, json};

use crate::effect::Effect;
use crate::model::{Model, Session};
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
fn descending_into_a_nested_collection_carries_the_parents_id() {
    let model = listing(vec![json!({"id": "u1", "username": "alice"})]);
    let opened = super::nav::open_selected(model).model;

    let with_object = update(
        opened,
        Msg::Item {
            token: 2,
            result: Ok(json!({"id": "u1", "username": "alice"})),
        },
    )
    .model;

    let sessions_tab = users()
        .tabs
        .iter()
        .find(|tab| tab.title() == "Sessions")
        .unwrap();
    let Transition { model, effects } = super::nested::enter_nested(with_object, sessions_tab);

    assert_eq!(model.resource().id, "user-sessions");
    assert_eq!(model.nav.crumbs(), vec!["Users", "alice"]);
    match effects.as_slice() {
        [Effect::Load { path, .. }] => {
            assert_eq!(path.as_str(), "/admin/realms/avsystem/users/u1/sessions");
        }
        other => panic!("expected a load, got {other:?}"),
    }
}

#[test]
fn saving_sends_the_whole_object_so_unknown_fields_survive() {
    let model = listing(vec![json!({"id": "u1", "username": "alice"})]);
    let opened = super::nav::open_selected(model).model;
    let token = opened.detail.as_ref().unwrap().token;

    let loaded = update(
        opened,
        Msg::Item {
            token,
            result: Ok(json!({
                "id": "u1",
                "username": "alice",
                "enabled": true,
                "fieldFromANewerKeycloak": {"keep": "me"}
            })),
        },
    )
    .model;

    // Move to `Enabled` and toggle it, then save.
    let on_enabled = (0..6).fold(loaded, |model, _| {
        update(model, key(KeyCode::Char('j'))).model
    });
    let toggled = update(on_enabled, key(KeyCode::Enter)).model;
    assert!(toggled.detail.as_ref().unwrap().is_dirty());

    let confirming = update(toggled, key(KeyCode::Char('w'))).model;
    let Transition { effects, .. } = update(confirming, key(KeyCode::Enter));

    match effects.as_slice() {
        [
            Effect::Call {
                body: Some(body), ..
            },
        ] => {
            assert_eq!(body["fieldFromANewerKeycloak"]["keep"], json!("me"));
            assert_eq!(body["enabled"], json!(false));
        }
        other => panic!("expected a save, got {other:?}"),
    }
}
