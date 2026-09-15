use std::time::Duration;

/// TLS handling, split out because an operator sometimes has to make a deliberate,
/// visible-in-the-UI compromise on a lab server.
#[derive(Debug, Clone, Default)]
pub struct TlsOptions {
    /// Skip certificate verification. Shown as a loud warning in the status bar whenever on.
    pub insecure: bool,
    /// Extra PEM bundle to trust, for internal CAs. The platform trust store is used too.
    pub ca_bundle: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct Timeouts {
    pub connect: Duration,
    pub request: Duration,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(10),
            // Realm exports and full LDAP syncs are genuinely slow.
            request: Duration::from_secs(120),
        }
    }
}
