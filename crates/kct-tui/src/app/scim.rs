//! "Can this client talk SCIM?" answered with one probe instead of a curl session.
//!
//! Verified against Keycloak 26.8.1: the realm switch off answers 404, a token without the
//! `<issuer>/scim/v2` audience answers 401, and a service account without `view-users` 403.

use std::sync::Arc;

use kct_core::jwt::Token;
use kct_core::path::{PathParams, endpoints as ep};
use kct_core::ports::Gateway;
use kct_core::{ApiPath, Error, Request, Secret};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use crate::msg::Msg;

use super::tokens::client_credentials;

pub(super) async fn probe(
    realm: String,
    client_id: String,
    secret_path: ApiPath,
    gateway: &Arc<dyn Gateway>,
    tx: &UnboundedSender<Msg>,
) {
    let Some(token) = client_credentials(&realm, &client_id, secret_path, gateway, tx).await else {
        return;
    };
    let Ok(root) = ep::SCIM.resolve(&PathParams::realm(&realm)) else {
        return;
    };

    let audience = expected_audience(&token);
    let mut lines = vec![
        format!("{:<16}{client_id}", "Client"),
        format!("{:<16}{root}", "Endpoint"),
        format!(
            "{:<16}{}",
            "Audience",
            audience_line(&token, audience.as_deref())
        ),
        String::new(),
    ];

    for (label, path, query) in [
        ("Service config", root.sub("ServiceProviderConfig"), vec![]),
        (
            "Users (read)",
            root.sub("Users"),
            vec![("count".to_owned(), "1".to_owned())],
        ),
    ] {
        let request = Request::get(path)
            .with_query(query)
            .with_bearer(token.clone());
        let line = match gateway.send(request).await {
            Ok(response) => ok_line(&response.body),
            Err(error) => diagnose(&error, audience.as_deref()),
        };
        lines.push(format!("{label:<16}{line}"));
    }

    let _ = tx.send(Msg::Text {
        title: format!("SCIM as {client_id}"),
        result: Ok(lines.join("\n") + "\n"),
    });
}

/// Keycloak wants the realm's SCIM root as an audience: the issuer plus `/scim/v2`.
fn expected_audience(token: &Secret) -> Option<String> {
    let issuer = Token::decode(token.expose()).ok()?.claims().issuer?;
    Some(format!("{issuer}/scim/v2"))
}

fn audience_line(token: &Secret, expected: Option<&str>) -> String {
    let Some(expected) = expected else {
        return "token is not a JWT; cannot check".to_owned();
    };
    let present = Token::decode(token.expose())
        .is_ok_and(|token| token.claims().audience.iter().any(|aud| aud == expected));
    if present {
        format!("ok ({expected})")
    } else {
        format!("MISSING {expected}")
    }
}

fn ok_line(body: &Value) -> String {
    match body.get("totalResults").and_then(Value::as_u64) {
        Some(total) => format!("ok, {total} users visible"),
        None => "ok".to_owned(),
    }
}

fn diagnose(error: &Error, audience: Option<&str>) -> String {
    match error {
        Error::Api { status: 404, .. } => "404: SCIM is off for this realm. Realm settings \
            ▸ General ▸ SCIM API (and the `scim-api` feature before 26.8)."
            .to_owned(),
        Error::Api { status: 401, .. } => format!(
            "401: the token lacks the audience {}. Run 'Prepare client for SCIM'.",
            audience.unwrap_or("<issuer>/scim/v2")
        ),
        Error::Api { status: 403, .. } => "403: the service account needs `view-users` to \
            read, `manage-users` to write (realm-management client roles)."
            .to_owned(),
        other => format!("failed: {other}"),
    }
}
