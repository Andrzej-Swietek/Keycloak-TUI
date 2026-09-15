//! A string that refuses to print itself.
//!
//! Client secrets, bind credentials and bearer tokens pass through this tool constantly. The
//! failure mode to design against is not malice but routine: a `dbg!`, a `tracing::debug!`,
//! an error message that embeds a response body. Wrapping the value makes the safe thing the
//! default and leaking it a deliberate act (`expose`).

use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Deliberately read the secret. Every call site is a place to ask "does this end up in a
    /// log?".
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// A stable, non-reversible fingerprint — enough to tell two secrets apart in a support
    /// conversation without revealing either.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        let hash = self.0.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |acc, byte| {
            (acc ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
        format!("{:08x}", hash & 0xffff_ffff)
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret(<redacted:{}>)", self.fingerprint())
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl From<String> for Secret {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Secret {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neither_debug_nor_display_can_leak_the_value() {
        let secret = Secret::new("hunter2");
        assert!(!format!("{secret:?}").contains("hunter2"));
        assert!(!format!("{secret}").contains("hunter2"));
        assert_eq!(secret.expose(), "hunter2");
    }

    #[test]
    fn fingerprints_distinguish_secrets_without_revealing_them() {
        let a = Secret::new("one");
        let b = Secret::new("two");
        assert_ne!(a.fingerprint(), b.fingerprint());
        assert_eq!(a.fingerprint(), Secret::new("one").fingerprint());
        assert!(!a.fingerprint().contains("one"));
    }
}
