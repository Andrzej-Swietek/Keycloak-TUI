use super::tests_support::{key, with_candidates};
use crossterm::event::KeyCode;
use kct_core::{Context, Method};
use serde_json::json;
use std::sync::Arc;

use crate::effect::Effect;
use crate::model::{Location, Model, Session};

use super::{Transition, update};

/// A user's realm role-mapping tab, which is where role assignment is reached from.
fn role_mappings_for(user: &str) -> Model {
    let users = kct_core::resources::by_id("users").unwrap();
    let mappings = kct_core::resources::by_id("role-mappings").unwrap();

    let model = Model::new(Session::new("https://kc", "avsystem", "admin"), users);
    Model {
        nav: model.nav.clone().pushed(Location {
            resource: mappings,
            context: Context::realm("avsystem")
                .with("ownerKind", "users")
                .with("owner", user),
            crumb: Arc::from("alice"),
        }),
        ..model
    }
}

fn opened_picker(action: &'static str) -> Transition {
    super::picker::open(role_mappings_for("u1"), action)
}

#[test]
fn joining_groups_sends_one_put_per_group() {
    let users = kct_core::resources::by_id("users").unwrap();
    let user_groups = kct_core::resources::by_id("user-groups").unwrap();
    let base = Model::new(Session::new("https://kc", "avsystem", "admin"), users);
    let model = Model {
        nav: base.nav.clone().pushed(Location {
            resource: user_groups,
            context: Context::realm("avsystem").with("user", "u1"),
            crumb: Arc::from("alice"),
        }),
        ..base
    };

    let Transition { model, .. } = super::picker::open(model, "join-group");
    let token = model.list.token + 1;
    let model = with_candidates(
        model,
        token,
        vec![
            json!({"id": "g1", "name": "sre", "path": "/sre"}),
            json!({"id": "g2", "name": "oncall", "path": "/sre/oncall"}),
        ],
    );

    let picked = update(update(model, key(KeyCode::Tab)).model, key(KeyCode::Tab)).model;
    let Transition { effects, .. } = update(picked, key(KeyCode::Enter));

    assert_eq!(effects.len(), 2, "group membership is one PUT per group");
    match &effects[0] {
        Effect::Call { method, path, .. } => {
            assert_eq!(*method, Method::Put);
            assert_eq!(path.as_str(), "/admin/realms/avsystem/users/u1/groups/g1");
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn a_stale_candidate_list_does_not_repopulate_a_closed_picker() {
    let Transition { model, .. } = opened_picker("assign-role");
    let closed = update(model, key(KeyCode::Esc)).model;

    let after = with_candidates(closed, 999, vec![json!({"id": "r1", "name": "x"})]);
    assert!(after.overlay.is_none());
}
