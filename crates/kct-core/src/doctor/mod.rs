//! Comparing the hand-written catalog against a live server.
//!
//! Four checks, each returning [`Finding`]s the caller collects into a [`Report`]:
//! [`check_feature_gates`] against `serverinfo`'s feature list, [`compare_fields`] and
//! [`compare_kinds`] against sampled objects, and [`compare_component`] against the property
//! contract `serverinfo` publishes per component type.
//!
//! Only [`Severity::Broken`] is fatal; see [`Severity`] for what each level means.

mod components;
mod fields;
mod finding;
mod kinds;

pub use components::compare as compare_component;
pub use fields::{compare as compare_fields, declared};
pub use finding::{Finding, Report, Severity};
pub use kinds::compare as compare_kinds;

use crate::capability::Requirement;
use crate::capability::ServerProfile;
use crate::resources;

/// Feature flags the catalog gates on that this server has never heard of.
///
/// A flag that vanished means the gate now hides a screen forever.
#[must_use]
pub fn check_feature_gates(profile: &ServerProfile) -> Vec<Finding> {
    fn flags(requirement: &Requirement) -> Vec<&'static str> {
        match requirement {
            Requirement::Feature(name) => vec![name],
            Requirement::All(parts) | Requirement::Any(parts) => {
                parts.iter().flat_map(flags).collect()
            }
            _ => Vec::new(),
        }
    }

    if !profile.source.is_complete() {
        return Vec::new();
    }

    resources::CATALOG
        .iter()
        .flat_map(|resource| {
            flags(&resource.requirement)
                .into_iter()
                .filter(|name| profile.feature(name).is_none())
                .map(|name| {
                    Finding::broken(
                        resource.id,
                        format!("gated on feature '{name}', which this server does not define"),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_kinds;
