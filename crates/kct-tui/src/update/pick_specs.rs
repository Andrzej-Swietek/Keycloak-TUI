use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, Result};

use crate::model::{Commit, Model};

use super::pick_spec::Spec;

/// Who the role mapping is for.
///
/// Two ways to know. Inside the role-mapping sub-list the parent pinned it into the context.
/// Everywhere else — the users list, an open user, a group's members — the highlighted row
/// *is* the owner, and requiring an operator to descend two levels first would be a poor
/// reason to refuse.
pub(super) fn owner(model: &Model) -> Option<(String, &'static str)> {
    let params = &model.context().params;
    if let (Some(kind), Some(id)) = (params.get("ownerKind"), params.get("owner")) {
        return Some((id.to_owned(), owner_kind(kind)));
    }

    let kind = match model.resource().id {
        "users" | "group-members" | "org-members" | "role-users" => "users",
        "groups" | "group-children" | "user-groups" | "role-groups" => "groups",
        _ => return None,
    };
    let (id, _) = super::actions::selection(model)?;
    Some((id, kind))
}

fn owner_kind(raw: &str) -> &'static str {
    if raw == "groups" { "groups" } else { "users" }
}

pub(super) fn assign_role(model: &Model, remove: bool) -> Result<Spec> {
    let (target, kind) = owner(model)
        .ok_or_else(|| kct_core::Error::config("open a user or group's role mapping tab first"))?;
    let params = PathParams::realm(&*model.session.realm)
        .with("ownerKind", kind)
        .with("owner", target);

    let assigned = ep::ROLE_MAPPINGS_REALM.resolve(&params)?;

    Ok(Spec::from(
        if remove {
            "Remove realm roles".to_owned()
        } else {
            "Assign realm roles".to_owned()
        },
        if remove {
            assigned.clone()
        } else {
            ep::ROLE_MAPPINGS_REALM_AVAILABLE.resolve(&params)?
        },
        &["/name"],
        Commit::Batch {
            method: if remove { Method::Delete } else { Method::Post },
            path: assigned,
            title: if remove {
                "roles removed".to_owned()
            } else {
                "roles assigned".to_owned()
            },
        },
    ))
}

pub(super) fn join_group(model: &Model) -> Result<Spec> {
    let user = user_id(model)?;
    let params = PathParams::realm(&*model.session.realm).with("user", user);

    Ok(Spec::from(
        "Join groups".to_owned(),
        ep::GROUPS.resolve(&PathParams::realm(&*model.session.realm))?,
        &["/path", "/name"],
        Commit::Each {
            method: Method::Put,
            base: ep::USER_GROUPS.resolve(&params)?,
            key: "id",
            title: "group membership updated".to_owned(),
        },
    ))
}

/// The parent screen decides whether a scope lands in the default or optional set.
pub(super) fn add_client_scope(model: &Model) -> Result<Spec> {
    let client = model
        .context()
        .params
        .get("client")
        .map(ToOwned::to_owned)
        .or_else(|| {
            // Straight from the clients list, the highlighted client is the target.
            (model.resource().id == "clients")
                .then(|| super::actions::selection(model).map(|(id, _)| id))
                .flatten()
        })
        .ok_or_else(|| kct_core::Error::config("select a client first"))?;
    let params = PathParams::realm(&*model.session.realm).with("client", client);

    let optional = model.resource().id == "client-optional-scopes";
    let base = if optional {
        ep::CLIENT_OPTIONAL_SCOPES.resolve(&params)?
    } else {
        ep::CLIENT_DEFAULT_SCOPES.resolve(&params)?
    };

    Ok(Spec::from(
        if optional {
            "Add optional client scopes".to_owned()
        } else {
            "Add default client scopes".to_owned()
        },
        ep::CLIENT_SCOPES.resolve(&PathParams::realm(&*model.session.realm))?,
        &["/name"],
        Commit::Each {
            method: Method::Put,
            base,
            key: "id",
            title: "client scope added".to_owned(),
        },
    )
    .requiring("/protocol", client_protocol(model))
    // A scope is default *or* optional, never both: Keycloak answers a PUT that would cross
    // the two with 204 and silently does nothing, so offer neither set.
    .excluding(vec![
        ep::CLIENT_DEFAULT_SCOPES.resolve(&params)?,
        ep::CLIENT_OPTIONAL_SCOPES.resolve(&params)?,
    ]))
}

/// The protocol of the client in view, which a scope must share to be attachable.
fn client_protocol(model: &Model) -> String {
    model
        .detail
        .as_ref()
        .map(|detail| &detail.original)
        .or_else(|| model.list.selected())
        .and_then(|client| client.get("protocol"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("openid-connect")
        .to_owned()
}

fn user_id(model: &Model) -> Result<String> {
    model
        .context()
        .params
        .get("user")
        .map(ToOwned::to_owned)
        .or_else(|| super::actions::selection(model).map(|(id, _)| id))
        .ok_or_else(|| kct_core::Error::config("select a user first"))
}

/// Required actions to send a user by e-mail.
pub(super) fn execute_actions_email(model: &Model) -> Result<Spec> {
    let (id, label) = super::actions::selection(model)
        .ok_or_else(|| kct_core::Error::config("select a user first"))?;
    let params = PathParams::realm(&*model.session.realm).with("user", id);

    Ok(Spec::from(
        format!("E-mail {label} these actions"),
        ep::USER_EXECUTE_ACTIONS_EMAIL.resolve(&params)?,
        &["/id"],
        Commit::Batch {
            method: Method::Put,
            path: ep::USER_EXECUTE_ACTIONS_EMAIL.resolve(&params)?,
            title: format!("e-mailed {label}"),
        },
    )
    .of(super::suggest::for_field(
        &kct_core::catalog::Field::new(
            "Required actions",
            "/requiredActions",
            kct_core::catalog::FieldKind::List,
        ),
        model.resource(),
        &model.session,
    )
    .into_iter()
    .map(|action| serde_json::json!({ "id": action }))
    .collect()))
}
