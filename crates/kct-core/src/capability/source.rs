/// Where a profile's facts came from, so the UI can say how much to trust a
/// "not available here" message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileSource {
    /// Full `GET /admin/serverinfo`.
    ServerInfo,
    /// OIDC discovery only: version inferred, feature flags unknown.
    Discovery,
    /// Nothing could be read; everything is permitted and nothing is promised.
    Assumed,
}

impl ProfileSource {
    #[must_use]
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::ServerInfo)
    }

    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::ServerInfo => "full server info",
            Self::Discovery => "discovery document only (serverinfo not readable)",
            Self::Assumed => "no server metadata available",
        }
    }
}
