use std::sync::Arc;

use kct_core::jwt::Token;
use kct_core::path::{PathParams, endpoints as ep};
use kct_core::ports::{Gateway, Rest, TokenSource};
use kct_core::{ApiPath, Method, Request, Secret, ports::Expect};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use crate::msg::{Msg, Notice};

use super::clipboard;

/// Decode the session's own token and present it as a readable report.
pub(super) async fn inspect(tokens: &Arc<dyn TokenSource>, tx: &UnboundedSender<Msg>) {
    let result = match tokens.access_token().await {
        Ok(token) => Ok(report(token.expose(), &tokens.describe())),
        Err(error) => Err(error),
    };

    let _ = tx.send(Msg::Text {
        title: "Access token".to_owned(),
        result,
    });
}

pub(super) async fn copy(tokens: &Arc<dyn TokenSource>, tx: &UnboundedSender<Msg>) {
    let notice = match tokens.access_token().await {
        Ok(token) => match clipboard::copy(token.expose()) {
            Ok(()) => Notice::success("access token copied"),
            Err(reason) => Notice::warning(format!("could not copy: {reason}")),
        },
        Err(error) => Notice::from_error("no token", &error),
    };
    let _ = tx.send(Msg::Notice(notice));
}

/// Fetch a confidential client's secret, exchange it for a token, and decode the result.
///
/// Two round trips in one effect on purpose: "what does this client actually get" is a single
/// question, and answering half of it is not useful.
pub(super) async fn for_client(
    realm: String,
    client_id: String,
    secret_path: ApiPath,
    gateway: &Arc<dyn Gateway>,
    tx: &UnboundedSender<Msg>,
) {
    let Some(token) = client_credentials(&realm, &client_id, secret_path, gateway, tx).await else {
        return;
    };
    let _ = tx.send(Msg::Text {
        title: format!("Token for {client_id}"),
        result: Ok(report(
            token.expose(),
            &format!("client_credentials as {client_id}"),
        )),
    });
}

/// A `client_credentials` token for this client, or `None` after telling the operator why.
pub(super) async fn client_credentials(
    realm: &str,
    client_id: &str,
    secret_path: ApiPath,
    gateway: &Arc<dyn Gateway>,
    tx: &UnboundedSender<Msg>,
) -> Option<Secret> {
    let secret = match gateway.get_json(secret_path, Vec::new()).await {
        Ok(body) => body
            .get("value")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        Err(error) => {
            let _ = tx.send(Msg::Notice(Notice::from_error(
                "cannot read the client secret (is the client confidential?)",
                &error,
            )));
            return None;
        }
    };

    let token_endpoint = ep::OIDC_TOKEN.resolve(&PathParams::realm(realm)).ok()?;
    let request = Request::new(Method::Post, token_endpoint)
        .with_form(vec![
            ("grant_type".to_owned(), "client_credentials".to_owned()),
            ("client_id".to_owned(), client_id.to_owned()),
            ("client_secret".to_owned(), secret),
        ])
        .expecting(Expect::Json)
        .anonymous();

    let token = gateway.send(request).await.and_then(|response| {
        response
            .body
            .get("access_token")
            .and_then(Value::as_str)
            .map(Secret::new)
            .ok_or_else(|| kct_core::Error::config("the token endpoint returned no access_token"))
    });
    match token {
        Ok(token) => Some(token),
        Err(error) => {
            let _ = tx.send(Msg::Notice(Notice::from_error(
                &format!("no token for {client_id}"),
                &error,
            )));
            None
        }
    }
}

/// Render a decoded token for reading. Does not verify the signature.
fn report(raw: &str, obtained_by: &str) -> String {
    let Ok(token) = Token::decode(raw) else {
        return format!("Obtained by: {obtained_by}\n\nOpaque token (not a JWT):\n{raw}\n");
    };
    let claims = token.claims();
    let line = |label: &str, value: String| format!("{label:<22}{value}\n");

    // Keycloak 26 issues lightweight access tokens for several built-in clients, including
    // admin-cli: no sub, no aud, no roles. Empty rows below would otherwise read as a bug.
    let lightweight = claims.subject.is_none() && token.payload.get("typ").is_some();

    [
        line("Obtained by", obtained_by.to_owned()),
        if lightweight {
            "Token type            lightweight — Keycloak omits sub, aud and roles; the \
             resource server resolves them by introspection\n"
                .to_owned()
        } else {
            String::new()
        },
        line("Signed with", token.signed_with()),
        line("Expiry", claims.expiry.describe()),
        line("Issuer", claims.issuer.unwrap_or_default()),
        line("Subject", claims.subject.unwrap_or_default()),
        line("Username", claims.preferred_username.unwrap_or_default()),
        line(
            "Authorized party",
            claims.authorized_party.unwrap_or_default(),
        ),
        line("Audience", claims.audience.join(", ")),
        line("Scope", claims.scope.join(" ")),
        line("Realm roles", claims.realm_roles.join(", ")),
        "\nSignature is NOT verified here; that is the resource server's job.\n".to_owned(),
        format!("\nHeader\n{}\n", kct_core::json::pretty(&token.header)),
        format!("\nPayload\n{}\n", kct_core::json::pretty(&token.payload)),
        format!("\nRaw\n{raw}\n"),
    ]
    .concat()
}
