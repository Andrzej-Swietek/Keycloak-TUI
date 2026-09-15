use std::sync::Arc;

use kct_core::catalog::{Action, Danger, Verb};
use kct_core::{Method, plan};
use serde_json::Value;

use crate::effect::Effect;
use crate::model::{Ask, Confirm, Model, Overlay, Prompt};
use crate::msg::Notice;
use crate::text_input::TextInput;

use super::{specials, transition::Transition};

/// Run one catalog action against the current selection.
#[must_use]
pub fn run(model: Model, action: &'static Action) -> Transition {
    let support = model.session.profile.evaluate(&action.requirement);
    if !support.is_permitted() {
        let why = support.reason().unwrap_or("not available").to_owned();
        return Transition::pure(
            model.noticed(Notice::warning(format!("{}: {why}", action.title))),
        );
    }

    if model.session.dry_run && action.danger != Danger::Safe {
        return Transition::pure(model.noticed(Notice::warning(format!(
            "dry-run is on; '{}' was not performed",
            action.title
        ))));
    }

    match action.verb {
        Verb::Create => create(model),
        Verb::Edit => Transition::pure(model.without_overlay()),
        Verb::Delete => delete(model, action),
        Verb::Post(suffix) => item_call(model, action, Method::Post, suffix),
        Verb::Fetch(suffix) => fetch(model, action, suffix),
        Verb::Export | Verb::Import | Verb::Duplicate | Verb::Special(_) => {
            specials::run(model, action)
        }
    }
}

fn create(model: Model) -> Transition {
    let resource = model.resource();
    let field = match resource.key.0 {
        "alias" => "alias",
        "realm" => "realm",
        _ if resource.id == "clients" => "clientId",
        _ if resource.id == "users" => "username",
        _ => "name",
    };

    Transition::pure(model.with_overlay(Overlay::Prompt(Prompt {
        title: Arc::from(format!("New {}", resource.singular.to_lowercase())),
        help: Arc::from(format!(
            "Creates a minimal {} with only '{field}' set; edit the rest afterwards.",
            resource.singular.to_lowercase()
        )),
        input: TextInput::default(),
        secret: false,
        ask: Ask::CreateNamed {
            resource: resource.id,
            field,
        },
    })))
}

fn delete(model: Model, action: &'static Action) -> Transition {
    let resource = model.resource();
    let Some((id, label)) = selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("nothing selected")));
    };

    let planned = match plan::item(resource, model.context(), &id) {
        Ok(planned) => planned,
        Err(error) => {
            return Transition::pure(model.noticed(Notice::from_error("cannot delete", &error)));
        }
    };

    let effect = Effect::Call {
        method: Method::Delete,
        path: planned.path,
        query: Vec::new(),
        body: None,
        title: format!("deleted {} '{label}'", resource.singular.to_lowercase()),
    };

    Transition::pure(model.with_overlay(Overlay::Confirm(Confirm {
        title: Arc::from(action.title),
        body: Arc::from(format!(
            "Delete {} '{label}'? This cannot be undone.",
            resource.singular.to_lowercase()
        )),
        danger: action.danger,
        expect: (action.danger == Danger::Destructive).then(|| Arc::from(label.as_str())),
        typed: TextInput::default(),
        on_confirm: Arc::new(vec![effect]),
    })))
}

fn item_call(
    model: Model,
    action: &'static Action,
    method: Method,
    suffix: &'static str,
) -> Transition {
    let resource = model.resource();
    let target = match selection(&model) {
        // Realm-wide verbs such as `logout-all` hang off the realm, not off a row.
        None if resource.is_singleton() || suffix.starts_with("logout") => None,
        None => return Transition::pure(model.noticed(Notice::warning("nothing selected"))),
        Some(found) => Some(found),
    };

    let path = match &target {
        Some((id, _)) => match plan::item(resource, model.context(), id) {
            Ok(planned) => planned.path.sub(suffix),
            Err(error) => {
                return Transition::pure(model.noticed(Notice::from_error("cannot run", &error)));
            }
        },
        None => kct_core::ApiPath::new(format!(
            "/admin/realms/{}/{suffix}",
            kct_core::path::encode_segment(&model.session.realm)
        )),
    };

    let label = target.map_or_else(|| model.session.realm.to_string(), |(_, label)| label);
    let effect = Effect::Call {
        method,
        path,
        query: Vec::new(),
        body: Some(Value::Object(serde_json::Map::new())),
        title: format!("{} · {label}", action.title.to_lowercase()),
    };

    Transition::pure(model.with_overlay(Overlay::Confirm(Confirm {
        title: Arc::from(action.title),
        body: Arc::from(format!("{} for '{label}'?", action.title)),
        danger: action.danger,
        expect: None,
        typed: TextInput::default(),
        on_confirm: Arc::new(vec![effect]),
    })))
}

fn fetch(model: Model, action: &'static Action, suffix: &'static str) -> Transition {
    let resource = model.resource();
    let Some((id, label)) = selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("nothing selected")));
    };

    match plan::item(resource, model.context(), &id) {
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
        Err(error) => Transition::pure(model.noticed(Notice::from_error("cannot fetch", &error))),
    }
}

/// The row an action applies to: the open object, else the highlighted row.
pub(super) fn selection(model: &Model) -> Option<(String, String)> {
    let resource = model.resource();

    if let Some(detail) = &model.detail {
        return Some((detail.id.clone(), detail.title.to_string()));
    }

    model.list.selected().and_then(|row| {
        resource
            .key_of(row)
            .map(|id| (id.to_owned(), resource.label_of(row)))
    })
}
