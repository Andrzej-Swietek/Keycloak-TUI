use std::collections::BTreeSet;

use serde_json::Value;

use crate::catalog::{Resource, Tab};
use crate::json;

use super::finding::Finding;

/// Objects needed before "the catalog declares this and no object has it" carries any signal.
const MIN_SAMPLES_FOR_UNSET: usize = 3;

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

/// Every pointer any screen over the same object renders.
///
/// Several screens reach one representation — a group through its own list, a user's groups
/// and a role's groups all resolve `GROUP` — and a key one of them renders is reachable from
/// all of them. Screens sharing only the generic `components` endpoint are excluded by their
/// differing type filter: they are different kinds behind one URL.
fn rendered_anywhere(resource: &Resource) -> BTreeSet<&'static str> {
    crate::resources::CATALOG
        .iter()
        .filter(|other| other.item == resource.item && other.fixed_params == resource.fixed_params)
        .flat_map(declared)
        .collect()
}

/// Compare what the catalog renders against what the server actually sent.
///
/// Two directions, neither fatal. A key the server sends that no screen renders is a
/// [`super::Severity::Missing`] coverage gap; the raw tab still reaches it. A pointer the catalog
/// declares that resolves on no sample is a [`super::Severity::Note`], and only once there are
/// enough samples for "unset everywhere" to mean anything — Keycloak omits unset optional
/// fields, so on one object it means nothing at all.
#[must_use]
pub fn compare(resource: &Resource, samples: &[Value]) -> Vec<Finding> {
    // A screen with no form has nothing to compare: its raw tab renders whatever arrives.
    if samples.is_empty() || declared(resource).is_empty() {
        return Vec::new();
    }

    let enough = samples.len() >= MIN_SAMPLES_FOR_UNSET;
    let stale = declared(resource)
        .into_iter()
        .filter(move |_| enough)
        // Pointers into a map the operator fills in themselves are absent by design.
        .filter(|pointer| !pointer.starts_with("/config/") && !pointer.starts_with("/attributes/"))
        .filter(|pointer| {
            samples
                .iter()
                .all(|sample| json::at(sample, pointer).is_none())
        })
        .map(|pointer| {
            Finding::note(
                resource.id,
                format!(
                    "form field '{pointer}' was on none of the {} sampled objects — unset, \
                     or renamed",
                    samples.len()
                ),
            )
        });

    let rendered = rendered_anywhere(resource);

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
        .filter(|key| !SERVER_ASSIGNED.contains(&key.as_str()))
        .filter(|key| !EDITED_ELSEWHERE.contains(&key.as_str()))
        .filter(|key| !SERVER_CATALOGUES.contains(&key.as_str()))
        .map(|key| {
            Finding::missing(
                resource.id,
                format!("server sends '{key}', which no form renders"),
            )
        });

    stale.chain(unrendered).collect()
}

/// Keys the server assigns and no form should offer to change.
const SERVER_ASSIGNED: &[&str] = &[
    "access",
    "adminPermissionsClient",
    "clientRole",
    "containerId",
    "disableableCredentialTypes",
    "id",
    "internalId",
    "notBefore",
    "origin",
    "parentId",
    "providerType",
    "self",
    "subGroupCount",
    "totp",
    "userProfileMetadata",
    "types",
];

/// What `serverinfo` publishes about the build rather than about this realm. Read through the
/// screens over them, or the raw tab.
const SERVER_CATALOGUES: &[&str] = &[
    "builtinProtocolMappers",
    "clientInstallations",
    "componentTypes",
    "cpuInfo",
    "cryptoInfo",
    "enums",
    "features",
    "identityProviders",
    "parameterizedScopeTypes",
    "passwordPolicies",
    "protocolMapperTypes",
    "providers",
    "socialProviders",
    "themes",
];

/// Keys carried on a representation whose own screen edits them.
const EDITED_ELSEWHERE: &[&str] = &[
    "adminEventsDetailsEnabled",
    "attributes",
    "adminEventsEnabled",
    "authenticationExecutions",
    "clientPolicies",
    "clientProfiles",
    "clientRoles",
    "defaultClientScopes",
    "enabledEventTypes",
    "eventsEnabled",
    "eventsExpiration",
    "eventsListeners",
    "groups",
    "optionalClientScopes",
    "protocolMappers",
    "realmRoles",
    "subGroups",
];
