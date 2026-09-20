//! Picking the type of an authorization policy or permission.
//!
//! The type is part of the URL a new one is created at, so it has to be settled before the
//! name. The list comes from the server, which is what makes a policy type added by a
//! deployed SPI provider selectable here without a code change.

use kct_core::Result;
use kct_core::path::Template;
use kct_core::path::{PathParams, endpoints as ep};

use crate::model::{Commit, Model};

use super::pick_spec::Spec;

pub(super) fn new_policy(model: &Model) -> Result<Spec> {
    kind(
        model,
        "policy",
        ep::CLIENT_AUTHZ_POLICY_PROVIDERS,
        "New policy",
        "policy",
    )
}

pub(super) fn new_permission(model: &Model) -> Result<Spec> {
    kind(
        model,
        "permission",
        ep::CLIENT_AUTHZ_PERMISSION_PROVIDERS,
        "New permission",
        "permission",
    )
}

fn kind(
    model: &Model,
    segment: &'static str,
    providers: Template,
    title: &'static str,
    singular: &'static str,
) -> Result<Spec> {
    let client = model
        .context()
        .params
        .get("client")
        .ok_or_else(|| kct_core::Error::config("open a client's authorization tab first"))?
        .to_owned();
    let params = PathParams::realm(&*model.session.realm).with("client", client);

    Ok(Spec::from(
        format!("{title}: choose a type"),
        providers.resolve(&params)?,
        &["/name", "/type"],
        Commit::Name {
            base: ep::CLIENT_AUTHZ_RESOURCE_SERVER
                .resolve(&params)?
                .sub(segment),
            from: "type",
            title,
            singular,
        },
    ))
}
