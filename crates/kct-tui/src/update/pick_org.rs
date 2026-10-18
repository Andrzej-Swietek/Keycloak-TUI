//! How an identity provider's members join the organization it is linked to (Keycloak 26.8+).

use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, Result};
use serde_json::json;

use crate::model::{Commit, Model};

use super::pick_spec::Spec;

fn org_params(model: &Model) -> Result<PathParams> {
    let org = model
        .context()
        .params
        .get("org")
        .ok_or_else(|| kct_core::Error::config("open an organization first"))?;
    Ok(PathParams::realm(&*model.session.realm).with("org", org.to_owned()))
}

/// The three link settings Keycloak accepts; `MANAGED` without auto-membership is a 400.
pub(super) fn set_membership(model: &Model) -> Result<Spec> {
    let (alias, _) = super::actions::selection(model)
        .ok_or_else(|| kct_core::Error::config("select an identity provider first"))?;
    let org = model.context().params.get("org").ok_or_else(|| {
        kct_core::Error::config("open an organization's identity providers first")
    })?;

    let params = PathParams::realm(&*model.session.realm).with("org", org.to_owned());
    let link = ep::ORGANIZATION_IDPS.resolve(&params)?.sub(&alias);
    let choice = |name: &str, kind: &str, auto: bool| json!({ "name": name, "link": { "membershipType": kind, "autoMembership": auto } });

    Ok(Spec::from(
        format!("Membership through '{alias}'"),
        link.clone(),
        &["/name"],
        Commit::Replace {
            method: Method::Put,
            path: link,
            base: json!({}),
            field: "",
            from: "link",
            title: format!("membership through '{alias}'"),
        },
    )
    .of(vec![
        choice(
            "Managed: joins on login, leaves when the link goes",
            "MANAGED",
            true,
        ),
        choice(
            "Unmanaged: joins on login, stays after unlinking",
            "UNMANAGED",
            true,
        ),
        choice(
            "Unmanaged, no auto-join: brokered users are not added",
            "UNMANAGED",
            false,
        ),
    ]))
}

/// Keycloak takes the type as a bare JSON string. A managed member is removed with the
/// identity provider link that brought them in; an unmanaged one stays.
pub(super) fn set_member_type(model: &Model) -> Result<Spec> {
    let (id, label) = super::actions::selection(model)
        .ok_or_else(|| kct_core::Error::config("select a member first"))?;
    let path = ep::ORGANIZATION_MEMBERS
        .resolve(&org_params(model)?)?
        .sub(&id)
        .sub("membership-type");

    Ok(Spec::from(
        format!("Membership type for '{label}'"),
        path.clone(),
        &["/name"],
        Commit::Replace {
            method: Method::Put,
            path,
            base: json!({}),
            field: "",
            from: "type",
            title: format!("membership type for '{label}'"),
        },
    )
    .of(vec![
        json!({ "name": "Managed: lifecycle follows the identity provider", "type": "MANAGED" }),
        json!({ "name": "Unmanaged: stays until removed by hand", "type": "UNMANAGED" }),
    ]))
}
