use std::collections::BTreeMap;

use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, Result};

use crate::model::{Commit, Model};

use super::pick_spec::Spec;
use super::pick_specs::owner;

/// Extra slots a chained picker carries from the step before it.
pub(super) type Extras = BTreeMap<&'static str, String>;

/// Step one of assigning a client role: which client's roles are we talking about.
pub(super) fn pick_client(model: &Model) -> Result<Spec> {
    owner(model).ok_or_else(|| kct_core::Error::config("select a user or group first"))?;

    Ok(Spec::from(
        "Assign client role — pick the client",
        ep::CLIENTS.resolve(&PathParams::realm(&*model.session.realm))?,
        &["/clientId", "/name"],
        Commit::Chain {
            next: "assign-client-role",
            slot: "client",
            key: "id",
        },
    ))
}

/// Step two: the roles of the client chosen a moment ago.
pub(super) fn assign_client_role(model: &Model, extras: &Extras) -> Result<Spec> {
    let (target, kind) =
        owner(model).ok_or_else(|| kct_core::Error::config("select a user or group first"))?;
    let client = extras
        .get("client")
        .ok_or_else(|| kct_core::Error::config("no client chosen"))?;

    let params = PathParams::realm(&*model.session.realm)
        .with("ownerKind", kind)
        .with("owner", target)
        .with("client", client.clone());

    Ok(Spec::from(
        "Assign client roles",
        ep::ROLE_MAPPINGS_CLIENT_AVAILABLE.resolve(&params)?,
        &["/name"],
        Commit::Batch {
            method: Method::Post,
            path: ep::ROLE_MAPPINGS_CLIENT.resolve(&params)?,
            title: "client roles assigned".to_owned(),
        },
    ))
}

/// Roles a composite role grants transitively.
pub(super) fn add_composite(model: &Model) -> Result<Spec> {
    let (id, name) = super::actions::selection(model)
        .ok_or_else(|| kct_core::Error::config("select a role first"))?;

    // Realm roles are keyed by name in their URL but composites are addressed by id, which
    // the row carries.
    let role_id = model
        .detail
        .as_ref()
        .map(|detail| &detail.original)
        .or_else(|| model.list.selected())
        .and_then(|role| role.get("id"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&id)
        .to_owned();
    let params = PathParams::realm(&*model.session.realm).with("role", role_id);
    let composites = ep::ROLE_COMPOSITES.resolve(&params)?;

    Ok(Spec::from(
        format!("Roles associated with '{name}'"),
        ep::ROLES.resolve(&PathParams::realm(&*model.session.realm))?,
        &["/name"],
        Commit::Batch {
            method: Method::Post,
            path: composites.clone(),
            title: "associated roles added".to_owned(),
        },
    )
    .excluding(vec![composites]))
}

/// Change an authentication step's requirement.
///
/// Keycloak takes the whole execution back with the new value, and each step advertises the
/// choices it supports — a conditional sub-flow has different options from a form.
pub(super) fn set_requirement(model: &Model) -> Result<Spec> {
    let execution = model
        .detail
        .as_ref()
        .map(|detail| &detail.original)
        .or_else(|| model.list.selected())
        .ok_or_else(|| kct_core::Error::config("select a step first"))?
        .clone();

    let flow = model
        .context()
        .params
        .get("flow")
        .ok_or_else(|| kct_core::Error::config("open a flow's executions first"))?;

    let choices: Vec<serde_json::Value> = execution
        .get("requirementChoices")
        .and_then(serde_json::Value::as_array)
        .map(|choices| {
            choices
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(|choice| serde_json::json!({ "name": choice }))
                .collect()
        })
        .unwrap_or_default();
    if choices.is_empty() {
        return Err(kct_core::Error::config(
            "this step advertises no requirement choices",
        ));
    }

    let params = PathParams::realm(&*model.session.realm).with("flow", flow.to_owned());
    let name = execution
        .get("displayName")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("step")
        .to_owned();

    Ok(Spec::from(
        format!("Requirement for '{name}'"),
        ep::AUTH_FLOW_EXECUTIONS.resolve(&params)?,
        &["/name"],
        Commit::Replace {
            method: Method::Put,
            path: ep::AUTH_FLOW_EXECUTIONS.resolve(&params)?,
            base: execution,
            field: "requirement",
            from: "name",
            title: format!("requirement for '{name}'"),
        },
    )
    .of(choices))
}
