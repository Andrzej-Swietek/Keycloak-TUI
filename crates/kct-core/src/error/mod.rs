//! One error type crossing every port boundary.
//!
//! Errors are `Clone` so they can travel through the UI message bus, and carry enough
//! context to say which call failed and what to do about it.

mod auth;
mod hint;
mod method;

pub use auth::AuthError;
pub use method::Method;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("authentication failed: {0}")]
    Auth(#[from] AuthError),

    #[error("{method} {path} -> {status}: {message}")]
    Api {
        status: u16,
        method: Method,
        path: String,
        message: String,
        detail: Option<String>,
    },

    #[error("cannot reach {url}: {reason}")]
    Transport { url: String, reason: String },

    #[error("unreadable response from {path}: {reason}")]
    Decode { path: String, reason: String },

    /// The server genuinely does not offer this: a disabled feature or an older version.
    #[error("{what} is unavailable here: {why}")]
    Unsupported { what: String, why: String },

    #[error("configuration problem: {0}")]
    Config(String),

    #[error("operation cancelled")]
    Cancelled,

    #[error("{0}")]
    Other(String),
}

impl Error {
    #[must_use]
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other(message.into())
    }

    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    #[must_use]
    pub fn unsupported(what: impl Into<String>, why: impl Into<String>) -> Self {
        Self::Unsupported {
            what: what.into(),
            why: why.into(),
        }
    }

    #[must_use]
    pub const fn status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// A 401 mid-session almost always means the access token aged out.
    #[must_use]
    pub const fn is_unauthenticated(&self) -> bool {
        matches!(self.status(), Some(401))
    }

    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        matches!(self.status(), Some(404))
    }

    /// Whether repeating the identical request could plausibly succeed.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::Transport { .. } => true,
            Self::Api { status, .. } => matches!(*status, 429 | 502 | 503 | 504),
            _ => false,
        }
    }

    #[must_use]
    pub fn hint(&self) -> Option<&'static str> {
        hint::for_error(self)
    }
}

#[cfg(test)]
mod tests;
