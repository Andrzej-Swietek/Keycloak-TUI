//! Helpers shared by the picker test modules.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde_json::Value;
use std::sync::Arc;

use kct_core::Context;

use crate::model::{Location, Model, Session};
use crate::msg::Msg;

use super::{Transition, update};

pub(super) fn key(code: KeyCode) -> Msg {
    Msg::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

/// A user's realm role-mapping tab, which is where role assignment is reached from.
pub(super) fn role_mappings_for(user: &str) -> Model {
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

pub(super) fn opened_picker(action: &'static str) -> Transition {
    super::picker::open(role_mappings_for("u1"), action)
}

pub(super) fn with_candidates(model: Model, token: u64, options: Vec<Value>) -> Model {
    update(
        model,
        Msg::Options {
            token,
            result: Ok(Arc::from(options)),
        },
    )
    .model
}
