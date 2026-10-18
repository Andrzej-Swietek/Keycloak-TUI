use super::tests_support::{key, with_candidates};
use crossterm::event::KeyCode;
use kct_core::{Context, Method};
use serde_json::json;
use std::sync::Arc;

use crate::effect::Effect;
use crate::model::{Location, Model, Overlay, Session};

use super::{Transition, update};

#[test]
fn assigning_a_client_role_asks_which_client_first() {
    let users = kct_core::resources::by_id("users").unwrap();
    let model = Model {
        list: crate::model::List {
            rows: Arc::from(vec![json!({"id": "u1", "username": "alice"})]),
            ..crate::model::List::default()
        }
        .refiltered(users),
        ..Model::new(Session::new("https://kc", "acme", "admin"), users)
    };

    let Transition { model, effects } = super::picker::open(model, "pick-client-for-role");
    match effects.as_slice() {
        [Effect::Load { path, .. }] => assert_eq!(path.as_str(), "/admin/realms/acme/clients"),
        other => panic!("expected the client list, got {other:?}"),
    }

    // Choosing a client opens the second step against that client's available roles.
    let token = model.list.token + 1;
    let model = with_candidates(
        model,
        token,
        vec![json!({"id": "c1", "clientId": "grafana"})],
    );
    let Transition { effects, .. } = update(model, key(KeyCode::Enter));

    match effects.as_slice() {
        [Effect::Load { path, .. }] => assert_eq!(
            path.as_str(),
            "/admin/realms/acme/users/u1/role-mappings/clients/c1/available"
        ),
        other => panic!("expected the client's roles, got {other:?}"),
    }
}

#[test]
fn a_step_offers_only_the_requirements_it_supports() {
    let executions = kct_core::resources::by_id("flow-executions").unwrap();
    let model = Model {
        list: crate::model::List {
            rows: Arc::from(vec![json!({
                "id": "e1",
                "displayName": "Kerberos",
                "requirement": "DISABLED",
                "requirementChoices": ["REQUIRED", "ALTERNATIVE", "DISABLED"]
            })]),
            ..crate::model::List::default()
        }
        .refiltered(executions),
        ..Model::new(Session::new("https://kc", "acme", "admin"), executions)
    };
    let model = Model {
        nav: model.nav.clone().pushed(Location {
            resource: executions,
            context: Context::realm("acme").with("flow", "browser"),
            crumb: Arc::from("browser"),
        }),
        ..model
    };

    let Transition { model, effects } = super::picker::open(model, "set-execution-requirement");
    assert!(
        effects.is_empty(),
        "the choices travel with the step; no fetch needed"
    );

    let Some(Overlay::Picker(picker)) = &model.overlay else {
        panic!("picker did not open")
    };
    assert_eq!(picker.matches.len(), 3);

    // Confirming sends the whole execution back with only `requirement` replaced.
    let Transition { effects, .. } = update(model, key(KeyCode::Enter));
    match effects.as_slice() {
        [
            Effect::Call {
                method,
                path,
                body: Some(body),
                ..
            },
        ] => {
            assert_eq!(*method, Method::Put);
            assert_eq!(
                path.as_str(),
                "/admin/realms/acme/authentication/flows/browser/executions"
            );
            assert_eq!(body["requirement"], json!("REQUIRED"));
            assert_eq!(body["id"], json!("e1"), "the execution travels back whole");
        }
        other => panic!("expected one update, got {other:?}"),
    }
}
