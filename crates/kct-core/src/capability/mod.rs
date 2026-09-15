//! What this particular server can actually do.
//!
//! Screens are gated on a [`Requirement`] evaluated against a profile read from the live
//! server, never on the version this tool was built against. An *unknown* answer always
//! resolves to "let the operator try": degrading to a clear error from the server beats
//! hiding a feature that is really there.

mod catalogues;
mod evaluate;
mod feature;
mod parse;
mod requirement;
mod source;

pub use feature::{FeatureFlag, FeatureKind, features};
pub use requirement::{Requirement, Support};
pub use source::ProfileSource;

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value;

use crate::version::{self, Version};

/// A snapshot of one server's capabilities. Cheap to clone, shared across the whole UI.
/// Fold the two spellings Keycloak uses onto one key.
fn normalise(name: &str) -> String {
    name.to_ascii_lowercase().replace('_', "-")
}

#[derive(Debug, Clone)]
pub struct ServerProfile {
    pub version: Version,
    /// The version string exactly as reported, qualifier included.
    pub version_label: Arc<str>,
    pub source: ProfileSource,
    pub features: Arc<BTreeMap<String, FeatureFlag>>,
    /// SPI name to provider ids. How custom extensions become visible without this tool
    /// knowing anything about them in advance.
    pub providers: Arc<parse::Catalog>,
    pub component_types: Arc<parse::Catalog>,
    pub protocol_mapper_types: Arc<parse::Catalog>,
    /// The untouched payload, so the server-info screen can show what we did not model.
    pub raw: Arc<Value>,
}

impl Default for ServerProfile {
    fn default() -> Self {
        Self::assumed(Version::UNKNOWN)
    }
}

impl ServerProfile {
    /// A profile that knows nothing and therefore forbids nothing.
    #[must_use]
    pub fn assumed(version: Version) -> Self {
        Self {
            version_label: Arc::from(version.to_string()),
            version,
            source: ProfileSource::Assumed,
            features: Arc::new(BTreeMap::new()),
            providers: Arc::new(BTreeMap::new()),
            component_types: Arc::new(BTreeMap::new()),
            protocol_mapper_types: Arc::new(BTreeMap::new()),
            raw: Arc::new(Value::Null),
        }
    }

    #[must_use]
    pub fn from_discovery(version: Version) -> Self {
        Self {
            source: ProfileSource::Discovery,
            ..Self::assumed(version)
        }
    }

    /// Parse `GET /admin/serverinfo`. Every section is optional: a server that omits one
    /// leaves us less informed, never broken.
    #[must_use]
    pub fn from_server_info(raw: Value) -> Self {
        let reported = raw.pointer("/systemInfo/version").and_then(Value::as_str);

        Self {
            version: reported.map_or(Version::UNKNOWN, Version::parse),
            version_label: Arc::from(reported.unwrap_or("unknown")),
            source: ProfileSource::ServerInfo,
            features: Arc::new(parse::features(&raw)),
            providers: Arc::new(parse::providers(&raw)),
            component_types: Arc::new(parse::id_lists(&raw, "componentTypes")),
            protocol_mapper_types: Arc::new(parse::id_lists(&raw, "protocolMapperTypes")),
            raw: Arc::new(raw),
        }
    }

    /// Version with its qualifier: `26.6.1`, or `999.0.0 (SNAPSHOT)`.
    #[must_use]
    pub fn version_display(&self) -> String {
        if self.version.is_unknown() {
            return "unknown".to_owned();
        }
        match version::qualifier_of(&self.version_label) {
            Some(qualifier) => format!("{} ({qualifier})", self.version),
            None => self.version.to_string(),
        }
    }

    /// A short version badge, and why it is missing when it is.
    ///
    /// Keycloak 26.7 removed the `view-system` role: a realm-scoped admin still receives the
    /// rest of `serverinfo` — features, SPIs, component types — but the `systemInfo` section
    /// is omitted. Capability detection is therefore unaffected; only the version is hidden,
    /// and saying so beats printing "unknown".
    #[must_use]
    pub fn version_summary(&self) -> String {
        match (self.version.is_unknown(), self.source) {
            (false, _) => self.version_display(),
            (true, ProfileSource::ServerInfo) => "version hidden".to_owned(),
            (true, ProfileSource::Discovery) => "version unreadable".to_owned(),
            (true, ProfileSource::Assumed) => "not detected".to_owned(),
        }
    }

    /// Whether feature and provider detection actually worked, regardless of the version.
    #[must_use]
    pub fn knows_capabilities(&self) -> bool {
        self.source.is_complete() && !self.features.is_empty()
    }

    /// Look a feature up by name, tolerating how Keycloak spells it.
    ///
    /// The `--features=` flag takes kebab-case (`admin-fine-grained-authz-v2`), but
    /// `serverinfo` reports the enum name, which is `UPPER_SNAKE` on 26.7 and was kebab-case on
    /// earlier releases. Matching literally means every gate silently reads as "this feature
    /// does not exist" after an upgrade — so both spellings are normalised before comparison.
    #[must_use]
    pub fn feature(&self, name: &str) -> Option<&FeatureFlag> {
        self.features.get(name).or_else(|| {
            let wanted = normalise(name);
            self.features
                .iter()
                .find(|(reported, _)| normalise(reported) == wanted)
                .map(|(_, flag)| flag)
        })
    }

    #[must_use]
    pub fn has_provider(&self, spi: &str, id: &str) -> bool {
        self.providers
            .get(spi)
            .is_some_and(|ids| ids.iter().any(|candidate| candidate == id))
    }

    /// Features that are on but not fully supported. An SRE debugging an incident deserves
    /// to know the realm depends on a preview feature.
    #[must_use]
    pub fn risky_enabled_features(&self) -> Vec<&FeatureFlag> {
        self.features
            .values()
            .filter(|flag| flag.enabled && flag.kind.is_risky())
            .collect()
    }

    #[must_use]
    pub fn evaluate(&self, requirement: &Requirement) -> Support {
        match requirement {
            Requirement::Always => Support::Available,
            Requirement::MinVersion(min) => self.check_min_version(*min),
            Requirement::BelowVersion(max) => self.check_below_version(*max),
            Requirement::Feature(name) => self.check_feature(name),
            Requirement::Provider { spi, id } => self.check_provider(spi, id),
            Requirement::All(parts) => parts
                .iter()
                .map(|part| self.evaluate(part))
                .fold(Support::Available, Support::and),
            Requirement::Any(parts) => parts
                .iter()
                .map(|part| self.evaluate(part))
                .reduce(Support::or)
                .unwrap_or(Support::Available),
        }
    }
}

#[cfg(test)]
mod tests;
