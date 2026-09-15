use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use kct_core::error::AuthError;
use kct_core::ports::{Gateway, Request, Response};
use kct_core::{Error, Result, Secret};

use crate::auth::Authenticator;
use crate::config::Connection;

use super::transport::HttpTransport;

/// Retries for a transient failure on a *read*. Writes are never retried automatically: a
/// `POST` that timed out may well have succeeded, and creating an object twice is worse than
/// showing an error.
const READ_RETRIES: u32 = 2;

/// A ready-to-use Keycloak client.
#[derive(Debug)]
pub struct KeycloakClient {
    transport: Arc<HttpTransport>,
    auth: Authenticator<HttpTransport>,
    connection: Connection,
}

impl KeycloakClient {
    pub fn connect(connection: Connection) -> Result<Self> {
        let transport = Arc::new(HttpTransport::new(&connection)?);
        let auth = Authenticator::new(
            transport.clone(),
            &connection.auth_realm,
            connection.credentials.clone(),
        );

        Ok(Self {
            transport,
            auth,
            connection,
        })
    }

    #[must_use]
    pub const fn connection(&self) -> &Connection {
        &self.connection
    }

    /// Prove the credentials work, before the UI commits to a session.
    pub async fn authenticate(&self) -> Result<()> {
        self.auth.bearer().await.map(|_| ())
    }

    /// Time left on the current access token, for the status bar.
    pub async fn token_expiry(&self) -> Option<Duration> {
        self.auth
            .snapshot()
            .await
            .and_then(|session| session.expires_in())
    }

    /// The raw access token, for the token inspector and for copying into `curl`.
    pub async fn access_token(&self) -> Result<Secret> {
        self.auth.bearer().await
    }

    pub async fn sign_out(&self) {
        self.auth.forget().await;
    }

    /// Point at a different realm. Cheap: it changes no connection state and does not
    /// re-authenticate, because the admin token is realm-independent.
    #[must_use]
    pub fn switch_realm(&self, realm: impl Into<String>) -> Connection {
        self.connection.clone().with_realm(realm)
    }

    async fn send_once(&self, request: &Request) -> Result<Response> {
        if !request.authenticated {
            return self.transport.execute(request, None).await;
        }
        let bearer = self.auth.bearer().await?;
        self.transport.execute(request, Some(&bearer)).await
    }
}

#[async_trait]
impl Gateway for KeycloakClient {
    async fn send(&self, request: Request) -> Result<Response> {
        if self.connection.dry_run && request.method.is_mutating() {
            tracing::info!(
                method = %request.method,
                path = %request.path,
                "dry run: request suppressed"
            );
            return Err(Error::Config(format!(
                "dry-run is on: {} {} was not sent",
                request.method, request.path
            )));
        }

        let attempts = if request.method.is_mutating() {
            1
        } else {
            READ_RETRIES + 1
        };

        let mut last: Option<Error> = None;
        for attempt in 0..attempts {
            match self.send_once(&request).await {
                Ok(response) => return Ok(response),

                // The server says the token is no good — believe it, renew, and try once
                // more. This is the common case after a laptop comes back from sleep.
                Err(error) if error.is_unauthenticated() && attempt == 0 => {
                    tracing::debug!("got 401, renewing the access token");
                    self.auth.invalidate().await;
                    match self.send_once(&request).await {
                        Ok(response) => return Ok(response),
                        Err(retry_error) => {
                            if retry_error.is_unauthenticated() {
                                return Err(Error::Auth(AuthError::RefreshFailed(
                                    "the server rejected a freshly issued token".to_owned(),
                                )));
                            }
                            last = Some(retry_error);
                        }
                    }
                }

                Err(error) if error.is_retryable() && attempt + 1 < attempts => {
                    let backoff = Duration::from_millis(200 * u64::from(attempt + 1));
                    tracing::debug!(?backoff, "retrying after a transient failure");
                    tokio::time::sleep(backoff).await;
                    last = Some(error);
                }

                Err(error) => return Err(error),
            }
        }

        Err(last.unwrap_or_else(|| Error::other("request failed with no reported cause")))
    }

    fn base_url(&self) -> &str {
        self.transport.base_url()
    }
}

/// The token tools need to see the credential the session is actually presenting.
#[async_trait]
impl kct_core::ports::TokenSource for KeycloakClient {
    async fn access_token(&self) -> Result<Secret> {
        self.auth.bearer().await
    }

    fn describe(&self) -> String {
        self.connection.credentials.describe()
    }
}
