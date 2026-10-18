use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde_json::json;
use std::sync::Arc;

use crate::model::{AttributeShape, Attributes, Detail, Model, Overlay, Session};
use crate::msg::Msg;

use super::update;

fn key(code: KeyCode) -> Msg {
    Msg::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn typed(model: Model, text: &str) -> Model {
    text.chars()
        .fold(model, |model, c| update(model, key(KeyCode::Char(c))).model)
}

fn editing_user_attributes() -> Model {
    let users = kct_core::resources::by_id("users").unwrap();
    let detail = Detail {
        original: json!({"id": "u1", "attributes": {"department": ["platform"]}}),
        draft: json!({"id": "u1", "attributes": {"department": ["platform"]}}),
        loading: false,
        ..Detail::loading("u1", "alice", 1)
    };

    let model = Model {
        detail: Some(detail.clone()),
        ..Model::new(Session::new("https://kc", "acme", "admin"), users)
    };

    let editor = Attributes::open(
        "Attributes",
        "/attributes",
        AttributeShape::ListValued,
        kct_core::json::at(&detail.draft, "/attributes"),
        Arc::from(Vec::new()),
        false,
    );
    model.with_overlay(Overlay::Attributes(editor))
}

#[test]
fn existing_attributes_are_listed_as_editable_rows() {
    let Some(Overlay::Attributes(editor)) = &editing_user_attributes().overlay else {
        panic!("editor did not open")
    };

    assert_eq!(editor.rows.len(), 1);
    assert_eq!(
        editor.rows[0],
        ("department".to_owned(), "platform".to_owned())
    );
}

#[test]
fn a_new_pair_can_be_typed_in_and_lands_in_the_draft() {
    let model = update(editing_user_attributes(), key(KeyCode::Char('a'))).model;
    let model = typed(model, "costCentre");
    let model = update(model, key(KeyCode::Enter)).model;

    // Move to the value column and fill it in.
    let model = update(model, key(KeyCode::Tab)).model;
    let model = update(model, key(KeyCode::Enter)).model;
    let model = typed(model, "42");
    let model = update(model, key(KeyCode::Enter)).model;

    // Closing writes the map back.
    let closed = update(model, key(KeyCode::Esc)).model;
    let draft = &closed.detail.as_ref().unwrap().draft;

    assert_eq!(draft["attributes"]["costCentre"], json!(["42"]));
    assert_eq!(
        draft["attributes"]["department"],
        json!(["platform"]),
        "the existing pair survives"
    );
}

#[test]
fn list_valued_maps_split_a_comma_separated_value() {
    let clear = Msg::Key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));

    let model = update(editing_user_attributes(), key(KeyCode::Enter)).model;
    let model = typed(update(model, clear).model, "a, b, c");
    let model = update(model, key(KeyCode::Enter)).model;

    let closed = update(model, key(KeyCode::Esc)).model;
    assert_eq!(
        closed.detail.unwrap().draft["attributes"]["department"],
        json!(["a", "b", "c"]),
        "users take Map<String, List<String>>"
    );
}

#[test]
fn a_realm_map_stays_string_valued() {
    let editor = Attributes::open(
        "Attributes",
        "/attributes",
        AttributeShape::StringValued,
        Some(&json!({"frontendUrl": "https://sso.example.com"})),
        Arc::from(Vec::new()),
        false,
    );

    assert_eq!(
        editor.to_json()["frontendUrl"],
        json!("https://sso.example.com"),
        "realms take Map<String, String>; an array here is silently ignored"
    );
}

#[test]
fn deleting_a_row_removes_it_from_the_map() {
    let model = update(editing_user_attributes(), key(KeyCode::Char('d'))).model;
    let closed = update(model, key(KeyCode::Esc)).model;

    assert_eq!(closed.detail.unwrap().draft["attributes"], json!({}));
}

#[test]
fn blank_keys_are_dropped_rather_than_sent() {
    let model = update(editing_user_attributes(), key(KeyCode::Char('a'))).model;
    let model = update(model, key(KeyCode::Esc)).model;
    let closed = update(model, key(KeyCode::Esc)).model;

    let attributes = &closed.detail.unwrap().draft["attributes"];
    assert_eq!(attributes.as_object().unwrap().len(), 1);
}

#[test]
fn undeclared_keys_are_flagged_when_the_realm_would_discard_them() {
    let editor = Attributes::open(
        "Attributes",
        "/attributes",
        AttributeShape::ListValued,
        Some(&json!({"department": ["platform"], "email": ["a@b.c"]})),
        Arc::from(vec!["email".to_owned()]),
        true,
    );

    assert_eq!(editor.undeclared(), vec!["department"]);
}
