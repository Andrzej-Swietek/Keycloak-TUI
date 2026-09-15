use crossterm::event::{KeyCode, KeyEvent};
use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, plan, resources};
use serde_json::json;

use crate::effect::Effect;
use crate::model::{Ask, List, Model, Prompt};
use crate::msg::Notice;

use super::{actions, nav, overlay, transition::Transition};

#[must_use]
pub fn handle(model: Model, state: Prompt, key: KeyEvent) -> Transition {
    match key.code {
        KeyCode::Esc => Transition::pure(model.without_overlay()),
        KeyCode::Enter => submit(model.without_overlay(), state),
        _ => overlay::typing(model, state, key),
    }
}

fn submit(model: Model, state: Prompt) -> Transition {
    let answer = state.input.value().trim().to_owned();
    if answer.is_empty() {
        return Transition::pure(model);
    }

    match state.ask {
        Ask::Search => search(model, answer),
        Ask::CreateNamed { resource, field } => create(model, resource, field, answer),
        Ask::SetPassword { temporary } => set_password(model, &answer, temporary),
        Ask::SaveToFile { body } => Transition::with(
            model,
            Effect::WriteFile {
                path: answer.into(),
                contents: body,
            },
        ),
        Ask::ImportFrom => import(model, answer),
        Ask::CreateJson { resource } => {
            super::create_from::create_from_json(model, resource, &answer)
        }
        Ask::AddSubFlow => add_sub_flow(model, &answer),
    }
}

/// Add an empty sub-flow to the flow whose executions are open.
fn add_sub_flow(model: Model, alias: &str) -> Transition {
    let Some(flow) = model.context().params.get("flow").map(ToOwned::to_owned) else {
        return Transition::pure(
            model.noticed(Notice::warning("open a flow's executions tab first")),
        );
    };
    let params = PathParams::realm(&*model.session.realm).with("flow", flow);

    match ep::AUTH_FLOW_ADD_SUBFLOW.resolve(&params) {
        Ok(path) => Transition::with(
            model,
            Effect::Call {
                method: Method::Post,
                path,
                query: Vec::new(),
                body: Some(json!({
                    "alias": alias,
                    "type": "basic-flow",
                    "description": "",
                    "provider": "registration-page-form",
                })),
                title: format!("added sub-flow '{alias}'"),
            },
        ),
        Err(error) => Transition::pure(model.noticed(Notice::from_error("cannot add", &error))),
    }
}

fn search(model: Model, needle: String) -> Transition {
    // Paging resets: page 4 of the previous result set means nothing for a new query.
    let query = model.list.query.clone().with_search(needle);
    nav::reload(Model {
        list: List {
            query: kct_core::Query { first: 0, ..query },
            ..model.list
        },
        ..model
    })
}

fn create(
    model: Model,
    resource_id: &'static str,
    _field: &'static str,
    name: String,
) -> Transition {
    let Some(resource) = resources::by_id(resource_id) else {
        return Transition::pure(model);
    };

    let planned = match plan::list(resource, model.context(), &model.list.query) {
        Ok(planned) => planned,
        Err(error) => {
            return Transition::pure(model.noticed(Notice::from_error("cannot create", &error)));
        }
    };

    // The resource says what a valid new object looks like. Guessing produces 400s: a
    // component has no `enabled` field, a client scope needs a protocol, an LDAP mapper
    // needs its parent.
    let draft = resource.draft_new(&name, &model.context().params);
    let title = format!("created {} '{name}'", resource.singular.to_lowercase());

    let (method, body) =
        match resource.with_item((*model.list.envelope).clone(), &name, Some(draft.clone())) {
            Some(document) => (Method::Put, document),
            None => (Method::Post, draft),
        };

    Transition::with(
        model,
        Effect::Call {
            method,
            path: planned.path,
            query: Vec::new(),
            body: Some(body),
            title,
        },
    )
}

fn set_password(model: Model, password: &str, temporary: bool) -> Transition {
    let Some((id, label)) = actions::selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("no user selected")));
    };
    let params = PathParams::realm(&*model.session.realm).with("user", id);

    match ep::USER_RESET_PASSWORD.resolve(&params) {
        Ok(path) => Transition::with(
            model,
            Effect::Call {
                method: Method::Put,
                path,
                query: Vec::new(),
                body: Some(json!({
                    "type": "password",
                    "value": password,
                    "temporary": temporary,
                })),
                title: format!("password set for '{label}'"),
            },
        ),
        Err(error) => {
            Transition::pure(model.noticed(Notice::from_error("cannot set password", &error)))
        }
    }
}

/// Partial import from a file, with Keycloak's `SKIP` policy: objects that already exist are
/// left alone.
fn import(model: Model, source: String) -> Transition {
    const POLICY: &str = "SKIP";

    let target = match ep::PARTIAL_IMPORT.resolve(&PathParams::realm(&*model.session.realm)) {
        Ok(target) => target,
        Err(error) => {
            return Transition::pure(model.noticed(Notice::from_error("cannot import", &error)));
        }
    };

    Transition::with(
        model,
        Effect::ImportFile {
            source: source.into(),
            target,
            policy: POLICY,
        },
    )
}
