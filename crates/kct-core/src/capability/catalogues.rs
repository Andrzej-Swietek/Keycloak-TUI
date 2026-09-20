use serde_json::Value;

use super::ServerProfile;

/// What the server says it has installed: themes, providers, mapper types.
impl ServerProfile {
    /// Theme names the server offers for one surface (`login`, `account`, `admin`, `email`).
    ///
    /// Offering only themes that are actually installed is the difference between a typo
    /// being caught here and a realm rendering the fallback theme in production.
    #[must_use]
    pub fn themes(&self, surface: &str) -> Vec<String> {
        self.raw
            .pointer(&format!("/themes/{surface}"))
            .and_then(Value::as_array)
            .map(|themes| {
                themes
                    .iter()
                    .filter_map(|theme| theme.get("name").and_then(Value::as_str))
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Provider ids registered for an SPI, for offering only what is installed.
    #[must_use]
    pub fn provider_ids(&self, spi: &str) -> Vec<String> {
        self.providers
            .get(spi)
            .map(|ids| ids.to_vec())
            .unwrap_or_default()
    }

    /// Password policy ids the server implements.
    #[must_use]
    pub fn password_policies(&self) -> Vec<String> {
        self.raw
            .get("passwordPolicies")
            .and_then(Value::as_array)
            .map(|policies| {
                policies
                    .iter()
                    .filter_map(|policy| policy.get("id").and_then(Value::as_str))
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Login event types the server can record.
    #[must_use]
    pub fn event_types(&self) -> Vec<String> {
        self.raw
            .pointer("/enums/eventType")
            .and_then(Value::as_array)
            .map(|types| {
                types
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Protocol mapper type ids for a protocol.
    #[must_use]
    pub fn mapper_types(&self, protocol: &str) -> Vec<String> {
        self.protocol_mapper_types
            .get(protocol)
            .map(|ids| ids.to_vec())
            .unwrap_or_default()
    }
}
