use std::collections::BTreeSet;

use serde_json::Value;

use crate::catalog::{Resource, Tab};
use crate::json;

use super::finding::Finding;

/// Fields the catalog claims a resource has, as JSON pointers.
#[must_use]
pub fn declared(resource: &Resource) -> BTreeSet<&'static str> {
    resource
        .tabs
        .iter()
        .filter_map(|tab| match tab {
            Tab::Form { fields, .. } => Some(*fields),
            _ => None,
        })
        .flatten()
        .map(|field| field.pointer)
        .collect()
}

/// Compare what the catalog renders against what the server actually sent.
///
/// Neither direction can prove a breakage on its own. Keycloak omits unset optional fields,
/// so a pointer that resolves on no sample may simply mean nobody filled it in — worth
/// looking at after an upgrade, not worth failing a build over. A key no form mentions is
/// likewise only a gap in coverage: the JSON tab still reaches it.
#[must_use]
pub fn compare(resource: &Resource, samples: &[Value]) -> Vec<Finding> {
    // A screen with no form has nothing to compare: its raw tab renders whatever arrives.
    if samples.is_empty() || declared(resource).is_empty() {
        return Vec::new();
    }

    let stale = declared(resource)
        .into_iter()
        // Pointers into a map the operator fills in themselves are absent by design.
        .filter(|pointer| !pointer.starts_with("/config/") && !pointer.starts_with("/attributes/"))
        .filter(|pointer| {
            samples
                .iter()
                .all(|sample| json::at(sample, pointer).is_none())
        })
        .map(|pointer| {
            Finding::missing(
                resource.id,
                format!(
                    "form field '{pointer}' was on none of the {} sampled objects — unset, \
                     or renamed",
                    samples.len()
                ),
            )
        });

    let rendered = declared(resource);

    // A form that reaches into nested data (`/systemInfo/version`, `/config/usersDn`) is not
    // comparable against top-level keys: it never intended to render them one-for-one.
    let covers = |key: &str| {
        let own = format!("/{key}");
        rendered
            .iter()
            .any(|pointer| *pointer == own || pointer.starts_with(&format!("{own}/")))
    };

    let unrendered = samples
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|object| object.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|key| !covers(key))
        .filter(|key| !IGNORED.contains(&key.as_str()))
        .map(|key| {
            Finding::missing(
                resource.id,
                format!("server sends '{key}', which no form renders"),
            )
        });

    stale.chain(unrendered).collect()
}

/// Keys every representation carries that are not worth a form control.
const IGNORED: &[&str] = &[
    "access",
    "clientRoles",
    "realmRoles",
    "subGroups",
    "protocolMappers",
    "origin",
    "self",
    "disableableCredentialTypes",
    "notBefore",
    "totp",
];
