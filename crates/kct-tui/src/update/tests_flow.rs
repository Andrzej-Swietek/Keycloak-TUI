use std::sync::Arc;

use kct_core::ports::Context;
use serde_json::json;

use crate::effect::Effect;
use crate::model::{Ask, Model, Overlay, Picker, Prompt, Session};
use crate::msg::Msg;
use crate::text_input::TextInput;

use super::{Transition, update};

fn on_executions() -> Model {
    let executions = kct_core::resources::by_id("flow-executions").unwrap();
    let model = Model::new(
        Session::new("https://kc.test", "avsystem", "admin"),
        executions,
    );
    Model {
        nav: crate::model::Nav::rooted(
            executions,
            Context::realm("avsystem").with("flow", "browser"),
        ),
        ..model
    }
}

#[test]
fn adding_a_step_offers_the_servers_authenticators() {
    let action = kct_core::resources::by_id("flow-executions")
        .unwrap()
        .actions
        .iter()
        .find(|action| action.id == "add-step")
        .expect("the flow a new screen creates has to be fillable");

    let Transition { model, effects } = super::actions::run(on_executions(), action);

    assert!(
        matches!(model.overlay, Some(Overlay::Picker(_))),
        "a picker of the available authenticators opens"
    );
    match effects.as_slice() {
        [Effect::Load { path, .. }] => assert_eq!(
            path.as_str(),
            "/admin/realms/avsystem/authentication/authenticator-providers"
        ),
        other => panic!("expected the provider list to be fetched, got {other:?}"),
    }
}

#[test]
fn confirming_the_picker_posts_one_step_per_choice() {
    let action = kct_core::resources::by_id("flow-executions")
        .unwrap()
        .actions
        .iter()
        .find(|action| action.id == "add-step")
        .unwrap();
    let opened = super::actions::run(on_executions(), action).model;
    let Some(Overlay::Picker(picker)) = opened.overlay.clone() else {
        panic!("no picker")
    };

    let loaded = Model {
        overlay: Some(Overlay::Picker(Box::new(Picker {
            options: Arc::from(vec![
                json!({"id": "auth-cookie", "displayName": "Cookie"}),
                json!({"id": "auth-spnego", "displayName": "Kerberos"}),
            ]),
            matches: Arc::from(vec![0, 1]),
            chosen: [0, 1].into_iter().collect(),
            ..*picker
        }))),
        ..opened
    };

    let Transition { effects, .. } = update(
        loaded,
        Msg::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        )),
    );

    let bodies: Vec<String> = effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::Call {
                path,
                body: Some(body),
                ..
            } => Some(format!("{} {body}", path.as_str())),
            _ => None,
        })
        .collect();

    assert_eq!(bodies.len(), 2, "one request per chosen step: {effects:?}");
    assert!(
        bodies[0].ends_with(r#"{"provider":"auth-cookie"}"#),
        "{bodies:?}"
    );
    assert!(bodies[0].contains("/authentication/flows/browser/executions/execution"));
}

#[test]
fn adding_a_sub_flow_posts_the_alias_that_was_typed() {
    let asking = Model {
        overlay: Some(Overlay::Prompt(Prompt {
            title: Arc::from("New sub-flow"),
            help: Arc::from(""),
            input: TextInput::new("2fa"),
            secret: false,
            ask: Ask::AddSubFlow,
        })),
        ..on_executions()
    };

    let Transition { effects, .. } = update(
        asking,
        Msg::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        )),
    );

    match effects.as_slice() {
        [
            Effect::Call {
                path,
                body: Some(body),
                ..
            },
        ] => {
            assert!(path.as_str().ends_with("/flows/browser/executions/flow"));
            assert_eq!(body["alias"], json!("2fa"));
            assert_eq!(body["type"], json!("basic-flow"));
        }
        other => panic!("expected one sub-flow creation, got {other:?}"),
    }
}

#[test]
fn moving_a_group_never_offers_the_group_itself() {
    use crate::model::List;

    let groups = kct_core::resources::by_id("groups").unwrap();
    let action = groups
        .actions
        .iter()
        .find(|action| action.id == "move")
        .expect("a group has to be reparentable");

    let rows = vec![json!({"id": "g1", "name": "platform", "path": "/platform"})];
    let model = Model {
        list: List {
            rows: Arc::from(rows.clone()),
            visible: Arc::from(vec![0]),
            envelope: Arc::new(json!(rows)),
            ..List::default()
        },
        ..Model::new(Session::new("https://kc.test", "avsystem", "admin"), groups)
    };

    let opened = super::actions::run(model, action).model;
    let Some(Overlay::Picker(picker)) = opened.overlay else {
        panic!("no picker opened")
    };

    assert!(
        picker.excluded.contains(&"g1".to_owned()),
        "a group cannot become its own parent: {:?}",
        picker.excluded
    );
    match &picker.commit {
        crate::model::Commit::EachUnder { suffix, body, .. } => {
            assert_eq!(*suffix, "children");
            assert_eq!(
                body["name"],
                json!("platform"),
                "Keycloak answers 400 to a move body without the name"
            );
        }
        other => panic!("expected a move, got {other:?}"),
    }
}
