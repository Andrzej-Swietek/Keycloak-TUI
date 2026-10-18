use super::tests_support::{key, opened_picker, with_candidates};
use crossterm::event::KeyCode;
use kct_core::Method;
use serde_json::{Value, json};

use crate::effect::Effect;
use crate::model::Overlay;

use super::{Transition, update};

#[test]
fn granting_offers_the_roles_the_owner_does_not_have_yet() {
    let Transition { model, effects } = opened_picker("assign-role");

    match effects.as_slice() {
        [Effect::Load { path, .. }] => assert_eq!(
            path.as_str(),
            "/admin/realms/acme/users/u1/role-mappings/realm/available",
            "granting must list what is missing, not what is already held"
        ),
        other => panic!("expected one load, got {other:?}"),
    }
    assert!(matches!(model.overlay, Some(Overlay::Picker(_))));
}

#[test]
fn revoking_offers_only_what_is_actually_assigned() {
    let Transition { effects, .. } = opened_picker("unassign-role");

    match effects.as_slice() {
        [Effect::Load { path, .. }] => assert_eq!(
            path.as_str(),
            "/admin/realms/acme/users/u1/role-mappings/realm"
        ),
        other => panic!("expected one load, got {other:?}"),
    }
}

#[test]
fn several_roles_can_be_granted_in_one_request() {
    let Transition { model, .. } = opened_picker("assign-role");
    let token = model.list.token + 1;
    let model = with_candidates(
        model,
        token,
        vec![
            json!({"id": "r1", "name": "view-users"}),
            json!({"id": "r2", "name": "manage-users"}),
            json!({"id": "r3", "name": "offline_access"}),
        ],
    );

    // Tick the first two, then confirm.
    let picked = update(update(model, key(KeyCode::Tab)).model, key(KeyCode::Tab)).model;
    let Transition { model, effects } = update(picked, key(KeyCode::Enter));

    assert!(model.overlay.is_none());
    match effects.as_slice() {
        [
            Effect::Call {
                method,
                path,
                body: Some(Value::Array(roles)),
                title,
                ..
            },
        ] => {
            assert_eq!(*method, Method::Post);
            assert_eq!(
                path.as_str(),
                "/admin/realms/acme/users/u1/role-mappings/realm"
            );
            assert_eq!(roles.len(), 2, "both ticked roles go in one call");
            assert_eq!(roles[0]["name"], json!("view-users"));
            assert!(title.contains("manage-users"));
        }
        other => panic!("expected one batched grant, got {other:?}"),
    }
}

#[test]
fn confirming_without_ticking_acts_on_the_highlighted_row() {
    let Transition { model, .. } = opened_picker("assign-role");
    let token = model.list.token + 1;
    let model = with_candidates(
        model,
        token,
        vec![json!({"id": "r1", "name": "view-users"})],
    );

    let Transition { effects, .. } = update(model, key(KeyCode::Enter));

    match effects.as_slice() {
        [
            Effect::Call {
                body: Some(Value::Array(roles)),
                ..
            },
        ] => {
            assert_eq!(roles.len(), 1);
            assert_eq!(roles[0]["name"], json!("view-users"));
        }
        other => panic!("expected a grant, got {other:?}"),
    }
}

#[test]
fn typing_filters_the_candidates() {
    let Transition { model, .. } = opened_picker("assign-role");
    let token = model.list.token + 1;
    let model = with_candidates(
        model,
        token,
        vec![
            json!({"id": "r1", "name": "view-users"}),
            json!({"id": "r2", "name": "offline_access"}),
        ],
    );

    let filtered = "offl"
        .chars()
        .fold(model, |model, c| update(model, key(KeyCode::Char(c))).model);

    let Some(Overlay::Picker(picker)) = &filtered.overlay else {
        panic!("picker closed")
    };
    assert_eq!(picker.matches.len(), 1);
    assert_eq!(
        picker.label_of(&picker.options[picker.matches[0]]),
        "offline_access"
    );
}
