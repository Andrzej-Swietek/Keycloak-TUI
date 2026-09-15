use super::tests_support::with_candidates;
use serde_json::json;
use std::sync::Arc;

use crate::effect::Effect;
use crate::model::{Model, Overlay, Session};
use crate::msg::Msg;

use super::{Transition, update};



/// The path an operator actually takes: highlight a user, press the action, pick a role.
/// It used to require descending into the role-mapping sub-list first, which nobody found.
#[test]
fn roles_can_be_assigned_straight_from_the_users_list() {
    let users = kct_core::resources::by_id("users").unwrap();
    let model = Model {
        list: crate::model::List {
            rows: Arc::from(vec![json!({"id": "u1", "username": "alice"})]),
            ..crate::model::List::default()
        }
        .refiltered(users),
        ..Model::new(Session::new("https://kc", "avsystem", "admin"), users)
    };

    let Transition { model, effects } = super::picker::open(model, "assign-role");

    assert!(matches!(model.overlay, Some(Overlay::Picker(_))));
    match effects.as_slice() {
        [Effect::Load { path, .. }] => assert_eq!(
            path.as_str(),
            "/admin/realms/avsystem/users/u1/role-mappings/realm/available"
        ),
        other => panic!("expected one load, got {other:?}"),
    }
}

#[test]
fn a_group_gets_its_own_owner_kind() {
    let groups = kct_core::resources::by_id("groups").unwrap();
    let model = Model {
        list: crate::model::List {
            rows: Arc::from(vec![json!({"id": "g1", "name": "sre", "path": "/sre"})]),
            ..crate::model::List::default()
        }
        .refiltered(groups),
        ..Model::new(Session::new("https://kc", "avsystem", "admin"), groups)
    };

    let Transition { effects, .. } = super::picker::open(model, "assign-role");
    match effects.as_slice() {
        [Effect::Load { path, .. }] => assert!(
            path.as_str().contains("/groups/g1/role-mappings/"),
            "a group must not be addressed as a user: {path}"
        ),
        other => panic!("expected one load, got {other:?}"),
    }
}

#[test]
fn scopes_already_attached_are_not_offered_again() {
    let clients = kct_core::resources::by_id("clients").unwrap();
    let model = Model {
        list: crate::model::List {
            rows: Arc::from(vec![
                json!({"id": "c1", "clientId": "grafana", "protocol": "openid-connect"}),
            ]),
            ..crate::model::List::default()
        }
        .refiltered(clients),
        ..Model::new(Session::new("https://kc", "avsystem", "admin"), clients)
    };

    let Transition { model, effects } = super::picker::open(model, "add-client-scope");
    assert_eq!(
        effects.len(),
        3,
        "candidates, plus the default and optional sets to subtract"
    );

    let token = model.list.token + 1;
    let model = with_candidates(
        model,
        token,
        vec![
            json!({"id": "s1", "name": "basic", "protocol": "openid-connect"}),
            json!({"id": "s2", "name": "phone", "protocol": "openid-connect"}),
            // A SAML scope on an OIDC client: Keycloak accepts the PUT and ignores it.
            json!({"id": "s3", "name": "role_list", "protocol": "saml"}),
        ],
    );
    let model = update(
        model,
        Msg::Exclusions {
            token,
            result: Ok(Arc::from(vec![json!({"id": "s1", "name": "basic"})])),
        },
    )
    .model;

    let Some(Overlay::Picker(picker)) = &model.overlay else {
        panic!("picker closed")
    };
    assert_eq!(
        picker.matches.len(),
        1,
        "`basic` is attached and `role_list` is the wrong protocol"
    );
    assert_eq!(picker.label_of(&picker.options[picker.matches[0]]), "phone");
}
