//! Moving a realm in and out as a file.

use std::sync::Arc;

use kct_core::catalog::Action;
use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, Result};

use crate::effect::Effect;
use crate::model::{Ask, Model, Overlay, Prompt};
use crate::text_input::TextInput;

use super::confirm::planned;
use super::transition::Transition;

pub(super) fn ask_import(model: Model) -> Transition {
    Transition::pure(model.with_overlay(Overlay::Prompt(Prompt {
        title: Arc::from("Partial import"),
        help: Arc::from(
            "Path to a realm export. Objects that already exist are SKIPPED, never overwritten.",
        ),
        input: TextInput::default(),
        secret: false,
        ask: Ask::ImportFrom,
    })))
}

pub(super) fn export(model: Model, action: &'static Action) -> Transition {
    let plan = |model: &Model| -> Result<Effect> {
        Ok(Effect::Fetch {
            method: Method::Post,
            path: ep::PARTIAL_EXPORT.resolve(&PathParams::realm(&*model.session.realm))?,
            query: vec![
                ("exportClients".to_owned(), "true".to_owned()),
                ("exportGroupsAndRoles".to_owned(), "true".to_owned()),
            ],
            body: None,
            title: format!("Export · {}", model.session.realm),
        })
    };
    planned(
        model,
        action,
        "Export this realm's clients, groups and roles? Secrets are redacted by Keycloak.",
        plan,
    )
}
