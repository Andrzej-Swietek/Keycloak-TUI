use std::time::Duration;

use kct_core::error::AuthError;
use kct_core::{Error, Result, Secret};
use serde_json::Value;
use tokio::time::Instant;

/// Renew this long before expiry: Keycloak access tokens default to five minutes, and a
/// request that starts valid must not finish expired.
const EXPIRY_SKEW: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct Session {
    pub access: Secret,
    pub refresh: Option<Secret>,
    access_expires: Option<Instant>,
    refresh_expires: Option<Instant>,
}

impl Session {
    /// A pre-issued token: it *is* the session, with no expiry we can see.
    #[must_use]
    pub fn from_bearer(token: Secret) -> Self {
        Self {
            access: token,
            refresh: None,
            access_expires: None,
            refresh_expires: None,
        }
    }

    /// Mark the access token as spent, keeping the refresh token.
    #[must_use]
    pub fn expired(self) -> Self {
        Self {
            access_expires: Some(Instant::now()),
            ..self
        }
    }

    #[must_use]
    pub(super) fn access_is_usable(&self) -> bool {
        self.access_expires
            .is_none_or(|deadline| Instant::now() + EXPIRY_SKEW < deadline)
    }

    #[must_use]
    pub(super) fn refresh_is_usable(&self) -> bool {
        self.refresh.is_some()
            && self
                .refresh_expires
                .is_none_or(|deadline| Instant::now() + EXPIRY_SKEW < deadline)
    }

    /// Seconds until the access token expires, for the status bar.
    #[must_use]
    pub fn expires_in(&self) -> Option<Duration> {
        self.access_expires
            .map(|deadline| deadline.saturating_duration_since(Instant::now()))
    }

    pub(super) fn from_token_response(body: &Value) -> Result<Self> {
        let access = body
            .get("access_token")
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| {
                Error::Auth(AuthError::OAuth {
                    error: "invalid_response".to_owned(),
                    description: Some("token endpoint returned no access_token".to_owned()),
                })
            })?;

        let lifetime = |field: &str| {
            body.get(field)
                .and_then(Value::as_u64)
                .map(|seconds| Instant::now() + Duration::from_secs(seconds))
        };

        Ok(Self {
            access: Secret::new(access),
            refresh: body
                .get("refresh_token")
                .and_then(Value::as_str)
                .map(Secret::new),
            access_expires: lifetime("expires_in"),
            refresh_expires: lifetime("refresh_expires_in"),
        })
    }
}
