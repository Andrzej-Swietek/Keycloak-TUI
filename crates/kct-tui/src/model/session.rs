use std::sync::Arc;

use kct_core::capability::{ServerProfile, Support};
use serde_json::Value;

/// What the realm's declarative user profile does with attributes it does not declare.
///
/// Since Keycloak 24 the default is to **discard them silently**: a write returns 201 and the
/// attribute simply is not there afterwards. Knowing this up front is the difference between
/// a two-second explanation and an afternoon of confusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnmanagedAttributes {
    Unknown,
    Discarded,
    Allowed,
}

impl UnmanagedAttributes {
    #[must_use]
    pub fn from_user_profile(profile: &Value) -> Self {
        match profile
            .get("unmanagedAttributePolicy")
            .and_then(Value::as_str)
        {
            Some(_) => Self::Allowed,
            None if profile.is_object() => Self::Discarded,
            None => Self::Unknown,
        }
    }

    /// Attribute names the profile declares. Writes to these survive whatever
    /// [`UnmanagedAttributes`] policy is in force.
    #[must_use]
    pub fn declared(profile: &Value) -> Vec<String> {
        profile
            .get("attributes")
            .and_then(Value::as_array)
            .map(|attributes| {
                attributes
                    .iter()
                    .filter_map(|attribute| attribute.get("name").and_then(Value::as_str))
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub base_url: Arc<str>,
    pub realm: Arc<str>,
    pub realms: Arc<[String]>,
    pub profile: ServerProfile,
    /// Human description of who we are connected as.
    pub account: Arc<str>,
    pub dry_run: bool,
    pub insecure: bool,
    /// Set when the connection is plain HTTP to something that is not loopback.
    pub cleartext: Option<Arc<str>>,
    pub unmanaged_attributes: UnmanagedAttributes,
    /// Carried in from the command line; the model owns it from then on.
    pub prefs: super::Prefs,
    /// Realm-level opt-ins. A feature can be GA on the server and still switched off here,
    /// which is a 404 rather than a 403 and needs its own explanation.
    pub realm_flags: Arc<Value>,
    /// Attribute names the realm's user profile declares.
    pub declared_attributes: Arc<[String]>,
    /// Aliases of the realm's top-level authentication flows, which is the vocabulary every
    /// flow-binding field accepts and which no static list could know.
    pub auth_flows: Arc<[String]>,
}

impl Session {
    /// Whether a screen is usable here, taking both the server's features and the realm's
    /// own switches into account.
    #[must_use]
    pub fn support_for(&self, resource: &kct_core::catalog::Resource) -> Support {
        let from_server = self.profile.evaluate(&resource.requirement);
        if !from_server.is_permitted() {
            return from_server;
        }

        match realm_switch(resource.id) {
            Some(flag) if self.realm_flags.get(flag) == Some(&Value::Bool(false)) => {
                Support::Unavailable(format!(
                    "'{flag}' is off for realm '{}'; turn it on in Realm settings first",
                    self.realm
                ))
            }
            _ => from_server,
        }
    }
}

/// Screens a realm can switch off independently of the server's feature flags.
fn realm_switch(resource_id: &str) -> Option<&'static str> {
    match resource_id {
        "organizations" | "org-members" | "org-identity-providers" => Some("organizationsEnabled"),
        _ => None,
    }
}

impl Session {
    #[must_use]
    pub fn new(
        base_url: impl Into<Arc<str>>,
        realm: impl Into<Arc<str>>,
        account: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            realm: realm.into(),
            realms: Arc::from(Vec::new()),
            profile: ServerProfile::default(),
            account: account.into(),
            dry_run: false,
            insecure: false,
            cleartext: None,
            unmanaged_attributes: UnmanagedAttributes::Unknown,
            prefs: super::Prefs::default(),
            realm_flags: Arc::new(Value::Null),
            declared_attributes: Arc::from(Vec::new()),
            auth_flows: Arc::from(Vec::new()),
        }
    }

    /// Warnings that belong in the status bar at all times, not in a dismissible toast.
    #[must_use]
    pub fn standing_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.dry_run {
            warnings.push("DRY-RUN".to_owned());
        }
        if self.insecure {
            warnings.push("TLS UNVERIFIED".to_owned());
        }
        if self.cleartext.is_some() {
            warnings.push("CLEARTEXT".to_owned());
        }
        if self.profile.version.is_below_floor() {
            warnings.push(format!("KC {} UNSUPPORTED", self.profile.version));
        }
        if self.profile.version.is_nightly() {
            warnings.push("NIGHTLY BUILD".to_owned());
        }
        warnings
    }
}
