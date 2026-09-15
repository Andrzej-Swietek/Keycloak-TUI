//! Saving an open detail: picking a suggestion, and writing an embedded item back.

use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde_json::json;

use crate::effect::Effect;
use crate::model::{Model, Session};
use crate::msg::Msg;

use super::{Transition, update};

fn key(code: KeyCode) -> Msg {
    Msg::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

#[test]
fn picking_a_suggestion_is_what_gets_committed() {
    use crate::model::{Detail, FieldEdit};
    use crate::text_input::TextInput;

    // The field starts holding something the operator half-typed; the pick must win over it.
    let detail = Detail {
        editing: Some(FieldEdit::new(
            0,
            TextInput::new("key"),
            vec!["keycloak".to_owned(), "keycloak.v2".to_owned()],
        )),
        ..Detail::loading("master", "master", 1)
    };

    let realm = kct_core::resources::by_id("realm-settings").unwrap();
    let base = Model {
        detail: Some(detail),
        ..Model::new(Session::new("https://kc.test", "master", "admin"), realm)
    };

    let field = super::visible_fields(&base, &realm.tabs[0])[0];
    let highlighted = update(base, key(KeyCode::Tab)).model;
    assert_eq!(
        highlighted.detail.as_ref().unwrap().editing.as_ref().unwrap().value(),
        "keycloak"
    );

    let committed = update(highlighted, key(KeyCode::Enter)).model;
    let written = field
        .read(&committed.detail.as_ref().unwrap().draft)
        .and_then(|value| value.as_str().map(ToOwned::to_owned));

    assert_eq!(
        written.as_deref(),
        Some("keycloak"),
        "Enter must write the highlighted suggestion, not the text typed under it"
    );
}

#[test]
fn saving_an_embedded_item_writes_the_whole_document_back() {
    use crate::model::{Detail, List};

    // The user profile keeps its attributes inside one document. Editing one must put that
    // document back with only that element changed — the others, and the policy beside them,
    // are not ours to drop.
    let profile = json!({
        "attributes": [
            {"name": "username", "displayName": "${username}"},
            {"name": "email", "displayName": "${email}", "permissions": {"edit": ["admin"]}},
        ],
        "groups": [{"name": "user-metadata"}],
        "unmanagedAttributePolicy": "ENABLED",
    });
    let attributes = kct_core::resources::by_id("user-profile-attributes").unwrap();
    let edited = json!({"name": "email", "displayName": "E-mail", "permissions": {"edit": ["admin"]}});

    let model = Model {
        list: List {
            envelope: Arc::new(profile),
            ..List::default()
        },
        detail: Some(Detail {
            draft: edited,
            ..Detail::loaded("email", "email", json!({"name": "email"}))
        }),
        ..Model::new(Session::new("https://kc.test", "avsystem", "admin"), attributes)
    };

    let confirming = update(model, key(KeyCode::Char('w'))).model;
    let Transition { effects, .. } = update(confirming, key(KeyCode::Enter));

    match effects.as_slice() {
        [
            Effect::Call {
                path,
                body: Some(body),
                ..
            },
        ] => {
            assert_eq!(path.as_str(), "/admin/realms/avsystem/users/profile");
            assert_eq!(body["attributes"][1]["displayName"], json!("E-mail"));
            assert_eq!(body["attributes"][0]["name"], json!("username"));
            assert_eq!(body["groups"][0]["name"], json!("user-metadata"));
            assert_eq!(body["unmanagedAttributePolicy"], json!("ENABLED"));
        }
        other => panic!("expected one save of the whole document, got {other:?}"),
    }
}

#[test]
fn a_row_with_no_endpoint_of_its_own_opens_without_fetching() {
    // Members of a role are users; there is no GET /roles/{role}/users/{id} to ask for.
    let members = kct_core::resources::by_id("role-users").unwrap();
    assert!(
        members.item.is_some(),
        "a role member is a user and must open at the user's own URL"
    );

    for resource in kct_core::resources::CATALOG {
        if resource.item.is_none() {
            assert!(
                !resource.ops.has(kct_core::catalog::Ops::UPDATE) || resource.embeds_items(),
                "{} has no item endpoint but claims to be writable",
                resource.id
            );
        }
    }
}

#[test]
fn a_successful_save_leaves_the_object_open() {
    // `:w` is vim's save, not vim's close.
    let users = kct_core::resources::by_id("users").unwrap();
    let model = Model {
        detail: Some(crate::model::Detail::loaded("u1", "alice", json!({"id": "u1"}))),
        ..Model::new(Session::new("https://kc.test", "avsystem", "admin"), users)
    };

    let Transition { model, effects } = update(
        model,
        Msg::Done {
            title: "saved user 'alice'".to_owned(),
            result: Ok(None),
        },
    );

    assert!(model.detail.is_some(), "the operator was still editing it");
    assert!(
        effects.iter().any(|effect| matches!(
            effect,
            Effect::Load {
                fetch: crate::msg::Fetch::Item,
                ..
            }
        )),
        "the object is refetched so the form shows what the server stored: {effects:?}"
    );
}

#[test]
fn write_quit_returns_to_the_listing() {
    let users = kct_core::resources::by_id("users").unwrap();
    let model = Model {
        detail: Some(crate::model::Detail::loaded("u1", "alice", json!({"id": "u1"}))),
        ..Model::new(Session::new("https://kc.test", "avsystem", "admin"), users)
    }
    .closing_after_save();

    let Transition { model, .. } = update(
        model,
        Msg::Done {
            title: "saved user 'alice'".to_owned(),
            result: Ok(None),
        },
    );

    assert!(model.detail.is_none());
}
