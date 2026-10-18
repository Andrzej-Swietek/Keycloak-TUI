use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use kct_core::ports::Context;
use serde_json::json;

use crate::effect::Effect;
use crate::model::{Ask, Model, Overlay, Picker, Session};
use crate::msg::Msg;

use super::{Transition, update};

fn key(code: KeyCode) -> Msg {
    Msg::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn on_policies() -> Model {
    let policies = kct_core::resources::by_id("client-authz-policies").unwrap();
    Model {
        nav: crate::model::Nav::rooted(policies, Context::realm("acme").with("client", "c1")),
        ..Model::new(Session::new("https://kc.test", "acme", "admin"), policies)
    }
}

#[test]
fn creating_a_policy_asks_the_server_which_types_exist() {
    // Posting to the bare collection answers 500: the type is part of the URL. Reading the
    // list from the server is also what makes a type a deployed SPI provider adds selectable.
    let Transition { model, effects } = update(on_policies(), key(KeyCode::Char('n')));

    assert!(matches!(model.overlay, Some(Overlay::Picker(_))));
    match effects.as_slice() {
        [Effect::Load { path, .. }] => assert_eq!(
            path.as_str(),
            "/admin/realms/acme/clients/c1/authz/resource-server/policy/providers"
        ),
        other => panic!("expected the type list to be fetched, got {other:?}"),
    }
}

#[test]
fn the_chosen_type_becomes_the_path_the_name_is_posted_to() {
    let opened = update(on_policies(), key(KeyCode::Char('n'))).model;
    let Some(Overlay::Picker(picker)) = opened.overlay.clone() else {
        panic!("no picker")
    };

    // A policy type this build has never heard of, as a deployment's own provider would be.
    let loaded = Model {
        overlay: Some(Overlay::Picker(Box::new(Picker {
            options: Arc::from(vec![
                json!({"type": "group-hierarchy", "name": "Group Hierarchy"}),
            ]),
            matches: Arc::from(vec![0]),
            chosen: [0].into_iter().collect(),
            ..*picker
        }))),
        ..opened
    };

    let asking = update(loaded, key(KeyCode::Enter)).model;
    let Some(Overlay::Prompt(prompt)) = &asking.overlay else {
        panic!("expected a name prompt, got {:?}", asking.overlay)
    };
    assert!(prompt.help.contains("group-hierarchy"), "{}", prompt.help);
    assert!(matches!(prompt.ask, Ask::PostNamed { .. }));

    let named = Model {
        overlay: Some(Overlay::Prompt(crate::model::Prompt {
            input: crate::text_input::TextInput::new("subtree-admins"),
            ..prompt.clone()
        })),
        ..asking
    };
    let Transition { effects, .. } = update(named, key(KeyCode::Enter));

    match effects.as_slice() {
        [
            Effect::Call {
                path,
                body: Some(body),
                ..
            },
        ] => {
            assert!(
                path.as_str()
                    .ends_with("/authz/resource-server/policy/group-hierarchy"),
                "{}",
                path.as_str()
            );
            assert_eq!(body["name"], json!("subtree-admins"));
        }
        other => panic!("expected one creation, got {other:?}"),
    }
}
