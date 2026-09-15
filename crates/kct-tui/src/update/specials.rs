use std::sync::Arc;

use kct_core::catalog::{Action, Verb};
use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, Result, plan};
use serde_json::json;

use crate::effect::Effect;
use crate::model::{Ask, Model, Overlay, Prompt};
use crate::msg::Notice;
use crate::text_input::TextInput;

use super::confirm::{ask, planned};
use super::{actions, ldap, nav, transition::Transition};

/// Verbs the generic CRUD engine cannot serve, each needing Keycloak-specific knowledge.
#[must_use]
pub fn run(model: Model, action: &'static Action) -> Transition {
    let name = match action.verb {
        Verb::Special(name) => name,
        Verb::Export => "partial-export",
        Verb::Import => "partial-import",
        Verb::Duplicate => "copy-flow",
        _ => return Transition::pure(model),
    };

    match name {
        "switch-realm" => switch_realm(model),
        action @ ("assign-role"
        | "unassign-role"
        | "join-group"
        | "add-client-scope"
        | "pick-client-for-role"
        | "add-composite"
        | "set-execution-requirement"
        | "add-execution"
        | "move-group") => super::picker::open(model, action),
        "add-sub-flow" => super::pick_flow::ask_sub_flow(model),
        "reset-password" => ask_password(model),
        "partial-export" => super::transfer::export(model, action),
        "partial-import" => super::transfer::ask_import(model),
        "token-for-client" => super::tokens::token_for_client(model),
        "regenerate-secret" => item_post(model, action, "client-secret"),
        "service-account-user" => fetch_item(model, action, "service-account-user"),
        "unlock-user" => unlock(model),
        "clear-brute-force" => clear_lockouts(model, action),
        "ldap-test-connection" => ldap::handle_test(model, action, false),
        "ldap-test-authentication" => ldap::handle_test(model, action, true),
        "ldap-sync-full" => ldap::handle_sync(model, action, true),
        "ldap-sync-changed" => ldap::handle_sync(model, action, false),
        "ldap-remove-imported" => ldap::handle_maintenance(model, action, false),
        "ldap-unlink-users" => ldap::handle_maintenance(model, action, true),
        other => Transition::pure(model.noticed(Notice::warning(format!(
            "'{other}' is not wired up yet; the JSON tab can do it by hand"
        )))),
    }
}

/// Ask where the realm representation lives before touching anything.
fn switch_realm(model: Model) -> Transition {
    match actions::selection(&model) {
        Some((realm, _)) => nav::switch_realm(model, realm),
        None => Transition::pure(model.noticed(Notice::warning("no realm selected"))),
    }
}

fn ask_password(model: Model) -> Transition {
    if actions::selection(&model).is_none() {
        return Transition::pure(model.noticed(Notice::warning("no user selected")));
    }

    Transition::pure(model.with_overlay(Overlay::Prompt(Prompt {
        title: Arc::from("Set password"),
        help: Arc::from("Stored as a temporary credential: the user must change it at next login."),
        input: TextInput::default(),
        secret: true,
        ask: Ask::SetPassword { temporary: true },
    })))
}

fn unlock(model: Model) -> Transition {
    let Some((id, label)) = actions::selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("no user selected")));
    };
    let params = PathParams::realm(&*model.session.realm).with("user", id);

    match ep::USER_UNLOCK.resolve(&params) {
        Ok(path) => Transition::with(
            model,
            Effect::Call {
                method: Method::Delete,
                path,
                query: Vec::new(),
                body: None,
                title: format!("lockout cleared for '{label}'"),
            },
        ),
        Err(error) => Transition::pure(model.noticed(Notice::from_error("cannot unlock", &error))),
    }
}

fn clear_lockouts(model: Model, action: &'static Action) -> Transition {
    let plan = |model: &Model| -> Result<Effect> {
        Ok(Effect::Call {
            method: Method::Delete,
            path: ep::ATTACK_DETECTION.resolve(&PathParams::realm(&*model.session.realm))?,
            query: Vec::new(),
            body: None,
            title: "all brute-force lockouts cleared".to_owned(),
        })
    };
    planned(
        model,
        action,
        "Clear every brute-force lockout in this realm?",
        plan,
    )
}

fn item_post(model: Model, action: &'static Action, suffix: &'static str) -> Transition {
    let Some((id, label)) = actions::selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("nothing selected")));
    };
    let resource = model.resource();

    match plan::item(resource, model.context(), &id) {
        Ok(planned) => ask(
            model,
            action,
            &format!("{} for '{label}'?", action.title),
            Effect::Call {
                method: Method::Post,
                path: planned.path.sub(suffix),
                query: Vec::new(),
                body: Some(json!({})),
                title: format!("{} · {label}", action.title.to_lowercase()),
            },
        ),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(action.title, &error))),
    }
}

fn fetch_item(model: Model, action: &'static Action, suffix: &'static str) -> Transition {
    let Some((id, label)) = actions::selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("nothing selected")));
    };

    match plan::item(model.resource(), model.context(), &id) {
        Ok(planned) => Transition::with(
            model,
            Effect::Fetch {
                method: Method::Get,
                path: planned.path.sub(suffix),
                query: Vec::new(),
                body: None,
                title: format!("{} · {label}", action.title),
            },
        ),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(action.title, &error))),
    }
}

pub(super) fn current_object(model: &Model) -> Option<serde_json::Value> {
    model
        .detail
        .as_ref()
        .map(|detail| detail.original.clone())
        .or_else(|| model.list.selected().cloned())
}
