//! Decoding a JWT for inspection.
//!
//! **This does not verify anything.** Signature and audience checks are the resource
//! server's job; conflating the two is how tools end up trusting a token they only decoded.
//! Everything here is for looking at a token an operator already has.

mod claims;

pub use claims::{Claims, Expiry};

use base64::Engine;
use serde_json::Value;

use crate::error::{Error, Result};

/// A decoded, unverified JWT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub header: Value,
    pub payload: Value,
    /// Present when the token is a JWS; absent for an opaque or malformed one.
    pub signature: Option<String>,
}

impl Token {
    /// Split and base64-decode a compact JWT.
    ///
    /// Errs on anything that is not three base64url segments, which includes the opaque
    /// refresh tokens Keycloak also issues.
    pub fn decode(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        let parts: Vec<&str> = raw.split('.').collect();

        if parts.len() != 3 {
            return Err(Error::Decode {
                path: "token".to_owned(),
                reason: format!(
                    "not a compact JWT: expected 3 dot-separated segments, found {}",
                    parts.len()
                ),
            });
        }

        Ok(Self {
            header: segment(parts[0], "header")?,
            payload: segment(parts[1], "payload")?,
            signature: Some(parts[2].to_owned()).filter(|part| !part.is_empty()),
        })
    }

    #[must_use]
    pub fn claims(&self) -> Claims {
        Claims::from(&self.payload)
    }

    /// The header's `alg` and `kid`.
    #[must_use]
    pub fn signed_with(&self) -> String {
        let field = |name: &str| {
            self.header
                .get(name)
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_owned()
        };
        format!("{} (kid {})", field("alg"), field("kid"))
    }
}

fn segment(encoded: &str, what: &str) -> Result<Value> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|error| Error::Decode {
            path: what.to_owned(),
            reason: error.to_string(),
        })?;

    serde_json::from_slice(&bytes).map_err(|error| Error::Decode {
        path: what.to_owned(),
        reason: error.to_string(),
    })
}

#[cfg(test)]
mod tests;
