use kct_core::error::AuthError;
use kct_core::{Error, Secret};
use serde_json::Value;

use crate::config::Credentials;

use super::{Authenticator, TokenEndpoint};

impl<E: TokenEndpoint> Authenticator<E> {
    pub(super) fn refresh_form(&self, refresh: &Secret) -> Vec<(String, String)> {
        let mut form = vec![
            ("grant_type".to_owned(), "refresh_token".to_owned()),
            ("refresh_token".to_owned(), refresh.expose().to_owned()),
            (
                "client_id".to_owned(),
                self.credentials.client_id().to_owned(),
            ),
        ];
        if let Some(secret) = self.client_secret() {
            form.push(("client_secret".to_owned(), secret));
        }
        form
    }

    pub(super) fn credentials_form(&self) -> Vec<(String, String)> {
        match &self.credentials {
            Credentials::Password {
                client_id,
                username,
                password,
                client_secret,
                otp,
            } => {
                let mut form = vec![
                    ("grant_type".to_owned(), "password".to_owned()),
                    ("client_id".to_owned(), client_id.clone()),
                    ("username".to_owned(), username.clone()),
                    ("password".to_owned(), password.expose().to_owned()),
                ];
                form.extend(
                    client_secret
                        .iter()
                        .map(|s| ("client_secret".to_owned(), s.expose().to_owned())),
                );
                form.extend(
                    otp.iter()
                        .map(|code| ("totp".to_owned(), code.expose().to_owned())),
                );
                form
            }
            Credentials::ClientCredentials {
                client_id,
                client_secret,
            } => vec![
                ("grant_type".to_owned(), "client_credentials".to_owned()),
                ("client_id".to_owned(), client_id.clone()),
                (
                    "client_secret".to_owned(),
                    client_secret.expose().to_owned(),
                ),
            ],
            Credentials::Bearer(_) => Vec::new(),
        }
    }

    pub(super) fn client_secret(&self) -> Option<String> {
        match &self.credentials {
            Credentials::Password { client_secret, .. } => {
                client_secret.as_ref().map(|s| s.expose().to_owned())
            }
            Credentials::ClientCredentials { client_secret, .. } => {
                Some(client_secret.expose().to_owned())
            }
            Credentials::Bearer(_) => None,
        }
    }
}

/// Turn an OAuth error body into a typed error.
///
/// Keycloak answers a failed grant with `{"error": ..., "error_description": ...}`; mapping
/// the common cases keeps the UI from showing "400 Bad Request" when it could say "wrong
/// password".
#[must_use]
pub fn oauth_error(body: &Value, client_id: &str, realm: &str) -> Error {
    let code = body
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or("invalid_request");
    let description = body
        .get("error_description")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    match code {
        "invalid_grant" | "unauthorized_client" | "invalid_client" => {
            Error::Auth(AuthError::InvalidCredentials {
                realm: realm.to_owned(),
                client_id: client_id.to_owned(),
            })
        }
        _ => Error::Auth(AuthError::OAuth {
            error: code.to_owned(),
            description,
        }),
    }
}
