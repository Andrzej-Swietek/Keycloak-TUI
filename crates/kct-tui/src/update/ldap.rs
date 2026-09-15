use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, Result};
use serde_json::{Value, json};

use kct_core::catalog::Action;

use crate::effect::Effect;
use crate::model::Model;
use crate::msg::Notice;

use super::confirm::ask;
use super::specials::current_object;
use super::{actions, transition::Transition};

/// Lift the connection settings out of a component's `config` map into the flat body
/// `testLDAPConnection` takes.
pub(super) fn test(model: &Model, component: &Value, authenticate: bool) -> Result<Effect> {
    let setting = |key: &str| {
        component
            .pointer(&format!("/config/{key}/0"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };

    let body = json!({
        "action": if authenticate { "testAuthentication" } else { "testConnection" },
        "connectionUrl": setting("connectionUrl"),
        "bindDn": setting("bindDn"),
        "bindCredential": setting("bindCredential"),
        "useTruststoreSpi": setting("useTruststoreSpi"),
        "connectionTimeout": setting("connectionTimeout"),
        "startTls": setting("startTls"),
        "authType": setting("authType"),
        "componentId": component.get("id").and_then(Value::as_str).unwrap_or_default(),
    });

    Ok(Effect::Call {
        method: Method::Post,
        path: ep::LDAP_TEST_CONNECTION.resolve(&PathParams::realm(&*model.session.realm))?,
        query: Vec::new(),
        body: Some(body),
        title: if authenticate {
            "LDAP authentication succeeded".to_owned()
        } else {
            "LDAP connection succeeded".to_owned()
        },
    })
}

/// Trigger a user-storage sync. Keycloak selects full vs incremental with a query parameter.
pub(super) fn sync(model: &Model, component_id: &str, full: bool) -> Result<Effect> {
    let params = PathParams::realm(&*model.session.realm).with("component", component_id);
    let action = if full {
        "triggerFullSync"
    } else {
        "triggerChangedUsersSync"
    };

    Ok(Effect::Call {
        method: Method::Post,
        path: ep::LDAP_SYNC.resolve(&params)?,
        query: vec![("action".to_owned(), action.to_owned())],
        body: None,
        title: format!("{} sync started", if full { "full" } else { "incremental" }),
    })
}

/// Remove locally imported copies of federated users, or cut their link to the provider.
pub(super) fn maintenance(model: &Model, component_id: &str, unlink: bool) -> Result<Effect> {
    let params = PathParams::realm(&*model.session.realm).with("component", component_id);
    let template = if unlink {
        ep::LDAP_UNLINK_USERS
    } else {
        ep::LDAP_REMOVE_IMPORTED
    };

    Ok(Effect::Call {
        method: Method::Post,
        path: template.resolve(&params)?,
        query: Vec::new(),
        body: None,
        title: if unlink {
            "users unlinked from the provider".to_owned()
        } else {
            "imported users removed".to_owned()
        },
    })
}

pub(super) fn handle_test(model: Model, action: &'static Action, authenticate: bool) -> Transition {
    let Some(component) = current_object(&model) else {
        return Transition::pure(model.noticed(Notice::warning("open a provider first")));
    };
    match test(&model, &component, authenticate) {
        Ok(effect) => Transition::with(model, effect),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(action.title, &error))),
    }
}

pub(super) fn handle_sync(model: Model, action: &'static Action, full: bool) -> Transition {
    let Some((id, label)) = actions::selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("no provider selected")));
    };
    let effect = match sync(&model, &id, full) {
        Ok(effect) => effect,
        Err(error) => {
            return Transition::pure(model.noticed(Notice::from_error(action.title, &error)));
        }
    };
    ask(
        model,
        action,
        &format!("{} for '{label}'?", action.title),
        effect,
    )
}

pub(super) fn handle_maintenance(
    model: Model,
    action: &'static Action,
    unlink: bool,
) -> Transition {
    let Some((id, label)) = actions::selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("no provider selected")));
    };
    let effect = match maintenance(&model, &id, unlink) {
        Ok(effect) => effect,
        Err(error) => {
            return Transition::pure(model.noticed(Notice::from_error(action.title, &error)));
        }
    };
    ask(
        model,
        action,
        &format!("{} for '{label}'?", action.title),
        effect,
    )
}
