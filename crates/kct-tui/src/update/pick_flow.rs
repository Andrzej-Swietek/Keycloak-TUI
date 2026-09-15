//! Picking what to add to an authentication flow.

use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, Result};

use crate::model::{Commit, Model};

use super::pick_specs::Spec;

/// Steps that can be added to the open flow.
///
/// The flow is addressed by alias; the executions screen is opened with it in the `flow`
/// slot, so this errs when run from anywhere else.
pub(super) fn add_execution(model: &Model) -> Result<Spec> {
    let flow = model
        .context()
        .params
        .get("flow")
        .ok_or_else(|| kct_core::Error::config("open a flow's executions tab first"))?
        .to_owned();
    let params = PathParams::realm(&*model.session.realm).with("flow", flow.clone());

    Ok(Spec {
        title: format!("Add a step to {flow}"),
        options: None,
        require: Vec::new(),
        exclude: Vec::new(),
        without: Vec::new(),
        source: ep::AUTH_AUTHENTICATOR_PROVIDERS.resolve(&params)?,
        label: &["/displayName", "/id"],
        commit: Commit::EachBody {
            method: Method::Post,
            path: ep::AUTH_FLOW_ADD_EXECUTION.resolve(&params)?,
            field: "provider",
            from: "id",
            title: "step added".to_owned(),
        },
    })
}

use std::sync::Arc;

use crate::model::{Ask, Overlay, Prompt};
use crate::text_input::TextInput;

use super::transition::Transition;

/// Ask for the sub-flow's name. Keycloak addresses it by alias from then on.
pub(super) fn ask_sub_flow(model: Model) -> Transition {
    Transition::pure(model.with_overlay(Overlay::Prompt(Prompt {
        title: Arc::from("New sub-flow"),
        help: Arc::from("Alias for the nested flow; add its steps once it exists."),
        input: TextInput::default(),
        secret: false,
        ask: Ask::AddSubFlow,
    })))
}
