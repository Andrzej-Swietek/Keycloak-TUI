//! Obtaining and renewing the access token.
//!
//! State lives behind interior mutability so the whole API stays `&self`, letting one client
//! be shared across every concurrent task without a lock in the application layer.

mod grants;
mod session;

pub use grants::oauth_error;
pub use session::Session;

use std::sync::Arc;

use kct_core::error::AuthError;
use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Error, Result, Secret};
use serde_json::Value;
use tokio::sync::{Mutex, RwLock};

use crate::config::Credentials;

/// Anything that can produce a bearer token, so the token endpoint can be faked in tests.
#[async_trait::async_trait]
pub trait TokenEndpoint: Send + Sync + 'static {
    /// Post a form to the realm's token endpoint and return the parsed JSON body.
    async fn post_form(&self, path: String, form: Vec<(String, String)>) -> Result<Value>;
}

/// Holds the current session and knows how to get a new one.
#[derive(Debug)]
pub struct Authenticator<E: TokenEndpoint> {
    endpoint: Arc<E>,
    auth_realm: String,
    credentials: Credentials,
    session: RwLock<Option<Session>>,
    /// Serialises renewal so a burst of concurrent requests triggers one token call, not one
    /// per request — which on a brute-force-protected realm would otherwise look like an
    /// attack and lock the admin account out.
    renewal: Mutex<()>,
}

impl<E: TokenEndpoint> Authenticator<E> {
    pub fn new(endpoint: Arc<E>, auth_realm: impl Into<String>, credentials: Credentials) -> Self {
        let credentials_session = match &credentials {
            // A pre-issued token *is* the session; there is nothing to fetch.
            Credentials::Bearer(token) => Some(Session::from_bearer(token.clone())),
            _ => None,
        };

        Self {
            endpoint,
            auth_realm: auth_realm.into(),
            credentials,
            session: RwLock::new(credentials_session),
            renewal: Mutex::new(()),
        }
    }

    /// A usable access token, renewing if necessary.
    pub async fn bearer(&self) -> Result<Secret> {
        if let Some(session) = self.current().await {
            return Ok(session.access);
        }

        let _guard = self.renewal.lock().await;
        // Another task may have renewed while we waited for the lock.
        if let Some(session) = self.current().await {
            return Ok(session.access);
        }

        let session = self.renew().await?;
        let access = session.access.clone();
        *self.session.write().await = Some(session);
        Ok(access)
    }

    /// Drop the current access token, forcing a renewal on the next call. Used after a 401,
    /// which means the server disagrees with us about validity.
    pub async fn invalidate(&self) {
        let mut session = self.session.write().await;
        *session = session.take().map(Session::expired);
    }

    /// Discard everything, including the refresh token.
    pub async fn forget(&self) {
        *self.session.write().await = None;
    }

    pub async fn snapshot(&self) -> Option<Session> {
        self.session.read().await.clone()
    }

    async fn current(&self) -> Option<Session> {
        self.session
            .read()
            .await
            .clone()
            .filter(Session::access_is_usable)
    }

    async fn renew(&self) -> Result<Session> {
        let existing = self.session.read().await.clone();

        // Prefer the refresh token: it avoids re-sending the password and keeps the
        // authentication event count down.
        if let Some(session) = existing.filter(Session::refresh_is_usable) {
            let refresh = session
                .refresh
                .clone()
                .expect("checked by refresh_is_usable");
            match self.exchange(self.refresh_form(&refresh)).await {
                Ok(renewed) => return Ok(renewed),
                Err(error) if !self.credentials.is_refreshable() => return Err(error),
                Err(_) => {
                    // The refresh token was revoked or the session ended server-side; fall
                    // through to a full re-authentication rather than failing the request.
                    tracing::debug!("refresh failed, re-authenticating from credentials");
                }
            }
        }

        match &self.credentials {
            Credentials::Bearer(_) => Err(Error::Auth(AuthError::RefreshFailed(
                "a pre-issued bearer token cannot be renewed".to_owned(),
            ))),
            _ => self.exchange(self.credentials_form()).await,
        }
    }

    async fn exchange(&self, form: Vec<(String, String)>) -> Result<Session> {
        let path = ep::OIDC_TOKEN
            .resolve(&PathParams::realm(&self.auth_realm))?
            .as_str()
            .to_owned();

        let body = self.endpoint.post_form(path, form).await?;
        Session::from_token_response(&body)
    }
}

#[cfg(test)]
mod tests;
