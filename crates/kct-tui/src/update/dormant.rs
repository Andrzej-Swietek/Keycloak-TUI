//! The actions that need a Keycloak-specific request each.
//!
//! One handler per verb the catalog names; [`super::tests_actions`] holds every catalog entry
//! to having one.

use std::sync::Arc;

use kct_core::catalog::Action;
use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, plan};

use crate::effect::Effect;
use crate::model::{Ask, Model, Overlay, Prompt};
use crate::msg::Notice;
use crate::text_input::TextInput;

use super::actions::selection;
use super::confirm::ask;
use super::transition::Transition;

/// Ask for the copy's name; Keycloak addresses the new flow by that alias.
pub(super) fn copy_flow(model: Model, action: &'static Action) -> Transition {
    let Some((_, label)) = selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("select a flow first")));
    };
    let params = PathParams::realm(&*model.session.realm).with("flow", label.clone());

    match ep::AUTH_FLOW_COPY.resolve(&params) {
        Ok(path) => Transition::pure(model.with_overlay(Overlay::Prompt(Prompt {
            title: Arc::from(action.title),
            help: Arc::from(format!("Alias for the copy of '{label}'")),
            input: TextInput::default(),
            secret: false,
            ask: Ask::PostNamed {
                path,
                field: "newName",
                singular: "copied flow to",
            },
        }))),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(action.title, &error))),
    }
}

/// End one session. The listing reaches it through a user or a client, but Keycloak deletes a
/// session at the realm's own path.
pub(super) fn revoke_session(model: Model, action: &'static Action) -> Transition {
    let Some((id, label)) = selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("select a session first")));
    };
    let params = PathParams::realm(&*model.session.realm).with("session", id);

    match ep::SESSION_DELETE.resolve(&params) {
        Ok(path) => ask(
            model,
            action,
            &format!("End the session of '{label}'?"),
            Effect::call(
                Method::Delete,
                path,
                None,
                format!("ended session · {label}"),
            ),
        ),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(action.title, &error))),
    }
}

/// Invite someone to the open organization by e-mail address.
pub(super) fn invite_member(model: Model, action: &'static Action) -> Transition {
    let Some((id, label)) = selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("select an organization first")));
    };
    let params = PathParams::realm(&*model.session.realm).with("org", id);

    match ep::ORGANIZATION_MEMBERS.resolve(&params) {
        Ok(path) => Transition::pure(model.with_overlay(Overlay::Prompt(Prompt {
            title: Arc::from(action.title),
            help: Arc::from(format!("E-mail address to invite into '{label}'")),
            input: TextInput::default(),
            secret: false,
            ask: Ask::PostNamed {
                path: path.sub("invite-user"),
                field: "email",
                singular: "invited",
            },
        }))),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(action.title, &error))),
    }
}

/// Re-run one federation mapper against the directory.
pub(super) fn mapper_sync(model: Model, action: &'static Action) -> Transition {
    let Some((id, label)) = selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("select a mapper first")));
    };
    let Some(component) = model
        .context()
        .params
        .get("component")
        .map(ToOwned::to_owned)
    else {
        return Transition::pure(
            model.noticed(Notice::warning("open the provider's mappers tab first")),
        );
    };
    let params = PathParams::realm(&*model.session.realm)
        .with("component", component)
        .with("mapper", id);

    match ep::LDAP_MAPPER_SYNC.resolve(&params) {
        Ok(path) => ask(
            model,
            action,
            &format!("Run '{label}' against the directory?"),
            Effect::Call {
                method: Method::Post,
                path,
                query: vec![("direction".to_owned(), "fedToKeycloak".to_owned())],
                body: None,
                title: format!("synced · {label}"),
            },
        ),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(action.title, &error))),
    }
}

/// The adapter configuration a client's protocol calls for.
pub(super) fn adapter_config(model: Model, action: &'static Action) -> Transition {
    let Some((id, label)) = selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("select a client first")));
    };
    let provider = match super::specials::current_object(&model)
        .or_else(|| model.list.selected().cloned())
        .as_ref()
        .and_then(|client| client.get("protocol"))
        .and_then(serde_json::Value::as_str)
    {
        Some("saml") => "keycloak-saml",
        _ => "keycloak-oidc-keycloak-json",
    };

    match plan::item(model.resource(), model.context(), &id) {
        Ok(planned) => Transition::with(
            model,
            Effect::Fetch {
                method: Method::Get,
                path: planned
                    .path
                    .sub("installation")
                    .sub("providers")
                    .sub(provider),
                query: Vec::new(),
                body: None,
                title: format!("{} · {label}", action.title),
            },
        ),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(action.title, &error))),
    }
}
