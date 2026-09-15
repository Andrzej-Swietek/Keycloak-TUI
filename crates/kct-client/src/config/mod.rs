//! How to reach one Keycloak server.

mod credentials;
mod tls;

pub use credentials::Credentials;
pub use tls::{Timeouts, TlsOptions};

use kct_core::{Error, Result};

#[derive(Debug, Clone)]
pub struct Connection {
    /// Base URL, with no trailing slash and no `/admin` suffix.
    pub base_url: String,
    /// Realm whose token endpoint issues our access token — where the admin account lives,
    /// usually `master`.
    pub auth_realm: String,
    /// Realm being administered. Changing it never re-authenticates.
    pub realm: String,
    pub credentials: Credentials,
    pub tls: TlsOptions,
    pub timeouts: Timeouts,
    /// Refuse every mutating request and report what would have been sent.
    pub dry_run: bool,
}

impl Connection {
    /// Validate and normalise. Catching a malformed URL here gives a far better message than
    /// a connection error five screens later.
    pub fn new(
        base_url: impl Into<String>,
        auth_realm: impl Into<String>,
        realm: impl Into<String>,
        credentials: Credentials,
    ) -> Result<Self> {
        let base_url = base_url.into().trim().trim_end_matches('/').to_owned();

        if base_url.is_empty() {
            return Err(Error::config("server URL is empty"));
        }
        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err(Error::config(format!(
                "server URL must start with http:// or https:// (got '{base_url}')"
            )));
        }

        let realm = realm.into();
        let auth_realm = auth_realm.into();

        Ok(Self {
            base_url,
            auth_realm: if auth_realm.is_empty() {
                "master".to_owned()
            } else {
                auth_realm
            },
            realm: if realm.is_empty() {
                "master".to_owned()
            } else {
                realm
            },
            credentials,
            tls: TlsOptions::default(),
            timeouts: Timeouts::default(),
            dry_run: false,
        })
    }

    #[must_use]
    pub fn with_tls(self, tls: TlsOptions) -> Self {
        Self { tls, ..self }
    }

    #[must_use]
    pub fn with_dry_run(self, dry_run: bool) -> Self {
        Self { dry_run, ..self }
    }

    #[must_use]
    pub fn with_realm(self, realm: impl Into<String>) -> Self {
        Self {
            realm: realm.into(),
            ..self
        }
    }

    /// Plain HTTP to anything but localhost sends the admin password in clear text.
    #[must_use]
    pub fn cleartext_warning(&self) -> Option<String> {
        let is_local = ["http://localhost", "http://127.0.0.1", "http://[::1]"]
            .iter()
            .any(|prefix| self.base_url.starts_with(prefix));

        (self.base_url.starts_with("http://") && !is_local).then(|| {
            format!(
                "{} is plain HTTP: credentials and tokens travel unencrypted",
                self.base_url
            )
        })
    }
}

#[cfg(test)]
mod tests;
