use std::collections::BTreeSet;

use serde_json::Value;

use crate::capability::ServerProfile;
use crate::catalog::{Encoding, Resource, Tab};

use super::finding::Finding;

/// Check a component-backed screen against `serverinfo.componentTypes`.
///
/// A key the form renders and the provider does not advertise yields [`super::Severity::Note`]; a
/// key advertised and not rendered yields [`super::Severity::Missing`]. Neither is fatal:
/// `componentTypes` omits the generic storage settings — `cachePolicy`, the sync periods —
/// which work regardless.
#[must_use]
pub fn compare(
    resource: &Resource,
    profile: &ServerProfile,
    spi: &str,
    provider: &str,
) -> Vec<Finding> {
    let declared = server_properties(profile, spi, provider);
    if declared.is_empty() {
        return vec![Finding::note(
            resource.id,
            format!("server declares no properties for {spi}/{provider}; nothing to compare"),
        )];
    }

    let rendered: BTreeSet<&str> = resource
        .tabs
        .iter()
        .filter_map(|tab| match tab {
            Tab::Form { fields, .. } => Some(*fields),
            _ => None,
        })
        .flatten()
        .filter(|field| field.encoding == Encoding::ConfigList)
        .filter_map(|field| field.pointer.strip_prefix("/config/"))
        .collect();

    let gone = rendered
        .iter()
        .filter(|key| !declared.contains(**key))
        .map(|key| {
            Finding::note(
                resource.id,
                format!("config key '{key}' is not advertised by {provider} (may still work)"),
            )
        });

    let unoffered = declared
        .iter()
        .filter(|key| !rendered.contains(key.as_str()))
        .map(|key| {
            Finding::missing(
                resource.id,
                format!("{provider} accepts '{key}', which no form offers"),
            )
        });

    gone.chain(unoffered).collect()
}

fn server_properties(profile: &ServerProfile, spi: &str, provider: &str) -> BTreeSet<String> {
    profile
        .raw
        .pointer(&format!("/componentTypes/{spi}"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|entry| entry.get("id").and_then(Value::as_str) == Some(provider))
        .and_then(|entry| entry.get("properties"))
        .and_then(Value::as_array)
        .map(|properties| {
            properties
                .iter()
                .filter_map(|property| property.get("name").and_then(Value::as_str))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
