/// Feature flags this tool reasons about by name. Keycloak's own names, not ours.
pub mod features {
    pub const ORGANIZATION: &str = "organization";
    pub const ADMIN_FINE_GRAINED_AUTHZ: &str = "admin-fine-grained-authz";
    pub const ADMIN_FINE_GRAINED_AUTHZ_V2: &str = "admin-fine-grained-authz-v2";
    pub const DECLARATIVE_USER_PROFILE: &str = "declarative-user-profile";
    pub const TOKEN_EXCHANGE: &str = "token-exchange";
    pub const CLIENT_POLICIES: &str = "client-policies";
    pub const DYNAMIC_SCOPES: &str = "dynamic-scopes";
    pub const IMPERSONATION: &str = "impersonation";
    pub const DEVICE_FLOW: &str = "device-flow";
    pub const AUTHORIZATION: &str = "authorization";
    pub const SCIM: &str = "scim";
    pub const WORKFLOWS: &str = "workflows";
    pub const PASSKEYS: &str = "passkeys";
}

/// Keycloak's `Profile.Feature.Type`, kept open-ended so a new classifier cannot break us.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureKind {
    Default,
    DisabledByDefault,
    Deprecated,
    Preview,
    Experimental,
    Unknown(String),
}

impl FeatureKind {
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw.to_ascii_uppercase().as_str() {
            "DEFAULT" => Self::Default,
            "DISABLED_BY_DEFAULT" => Self::DisabledByDefault,
            "DEPRECATED" => Self::Deprecated,
            "PREVIEW" | "PREVIEW_DISABLED_BY_DEFAULT" => Self::Preview,
            "EXPERIMENTAL" => Self::Experimental,
            other => Self::Unknown(other.to_owned()),
        }
    }

    /// Whether relying on this feature deserves a warning in the UI.
    #[must_use]
    pub const fn is_risky(&self) -> bool {
        matches!(self, Self::Preview | Self::Experimental | Self::Deprecated)
    }

    #[must_use]
    pub const fn badge(&self) -> &'static str {
        match self {
            Self::Default => "",
            Self::DisabledByDefault => "off by default",
            Self::Deprecated => "deprecated",
            Self::Preview => "preview",
            Self::Experimental => "experimental",
            Self::Unknown(_) => "?",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FeatureFlag {
    pub name: String,
    pub label: Option<String>,
    pub kind: FeatureKind,
    pub enabled: bool,
}
