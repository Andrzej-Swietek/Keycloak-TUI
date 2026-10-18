//! Making a client able to call the realm's SCIM API.

use kct_core::catalog::Action;
use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, Result, plan};
use serde_json::{Value, json};

use crate::effect::Effect;
use crate::model::Model;
use crate::msg::Notice;

use super::confirm::ask;
use super::transition::Transition;

/// Keycloak 26.8 accepts a SCIM call only from a token whose audience is the realm's SCIM
/// root, and only from a confidential client: an audience mapper settles the first.
#[must_use]
pub fn prepare_client(model: Model, action: &'static Action) -> Transition {
    let Some((id, label)) = super::actions::selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("no client selected")));
    };

    match plan_mapper(&model, &id) {
        Ok((effect, audience)) => ask(
            model,
            action,
            &format!(
                "Add an audience mapper for {audience} to '{label}'?\n\nThe service account \
                 still needs `view-users` (read) or `manage-users` (write) from \
                 realm-management; 'Probe SCIM as this client' checks the result."
            ),
            effect,
        ),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(action.title, &error))),
    }
}

fn plan_mapper(model: &Model, id: &str) -> Result<(Effect, String)> {
    let planned = plan::item(model.resource(), model.context(), id)?;
    let root = ep::SCIM.resolve(&PathParams::realm(&*model.session.realm))?;
    let audience = format!("{}{root}", model.session.base_url.trim_end_matches('/'));

    let effect = Effect::call(
        Method::Post,
        planned.path.sub("protocol-mappers").sub("models"),
        Some(audience_mapper(&audience)),
        "SCIM audience added".to_owned(),
    );
    Ok((effect, audience))
}

fn audience_mapper(audience: &str) -> Value {
    json!({
        "name": "scim-audience",
        "protocol": "openid-connect",
        "protocolMapper": "oidc-audience-mapper",
        "config": {
            "included.custom.audience": audience,
            "access.token.claim": "true",
            "introspection.token.claim": "true",
            "id.token.claim": "false"
        }
    })
}
