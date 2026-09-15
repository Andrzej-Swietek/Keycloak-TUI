use kct_core::Secret;

/// How this tool proves who it is.
#[derive(Debug, Clone)]
pub enum Credentials {
    /// Resource-owner password grant. The usual interactive choice; needs *direct access
    /// grants* enabled on the client (`admin-cli` has it by default).
    Password {
        client_id: String,
        username: String,
        password: Secret,
        /// Set when the admin client is confidential.
        client_secret: Option<Secret>,
        /// One-time code, when the direct-grant flow asks for OTP.
        otp: Option<Secret>,
    },
    /// Service account. The right choice for automation: no human password, and the
    /// account's roles can be scoped down to exactly what the job needs.
    ClientCredentials {
        client_id: String,
        client_secret: Secret,
    },
    /// A token obtained elsewhere (`kcadm.sh`, CI, a vault). Cannot be refreshed, so the
    /// session ends when the token does.
    Bearer(Secret),
}

impl Credentials {
    #[must_use]
    pub fn client_id(&self) -> &str {
        match self {
            Self::Password { client_id, .. } | Self::ClientCredentials { client_id, .. } => {
                client_id
            }
            Self::Bearer(_) => "(pre-issued token)",
        }
    }

    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Password {
                username,
                client_id,
                ..
            } => format!("{username} via {client_id}"),
            Self::ClientCredentials { client_id, .. } => format!("service account {client_id}"),
            Self::Bearer(_) => "pre-issued bearer token".to_owned(),
        }
    }

    #[must_use]
    pub const fn is_refreshable(&self) -> bool {
        !matches!(self, Self::Bearer(_))
    }
}
