//! The command palette.
//!
//! Keycloak's admin surface is wide enough that menu-hunting is the real cost of using it.
//! Because every screen and verb is declared in the catalog, the palette is *derived* rather
//! than maintained: a new resource appears here the moment it is added, keywords included.

mod command;
mod entry;
mod ex;

pub use command::{Command, Global};
pub use entry::Entry;
pub use ex::Ex;

use kct_core::capability::ServerProfile;
use kct_core::catalog::{Action, Resource};
use kct_core::resources;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Build the full palette for the current context.
///
/// `current` is the resource in view, if any: its actions are offered, and offered first,
/// because "the thing I am looking at" is what an operator usually means.
#[must_use]
pub fn catalog(
    profile: &ServerProfile,
    realms: &[String],
    current: Option<&'static Resource>,
) -> Vec<Entry> {
    let current_actions = current.into_iter().flat_map(|resource| {
        resource.actions.iter().map(move |action: &'static Action| {
            Entry::new(
                Command::Run {
                    resource: resource.id,
                    action: action.id,
                },
                action.title.to_owned(),
                format!("{} · this view", resource.plural),
                &action.keywords.join(" "),
            )
            .gated(&profile.evaluate(&action.requirement))
        })
    });

    let screens = resources::top_level().map(|resource| {
        Entry::new(
            Command::Open(resource.id),
            format!("Go to {}", resource.plural),
            resource.summary.to_owned(),
            &format!(
                "{} {} {}",
                resource.section.title(),
                resource.singular,
                resource.keywords.join(" ")
            ),
        )
        .gated(&profile.evaluate(&resource.requirement))
    });

    // Actions of *other* resources are reachable too, so "delete user" works from anywhere.
    let other_actions = resources::top_level()
        .filter(move |resource| current.is_none_or(|open| open.id != resource.id))
        .flat_map(|resource| {
            resource.actions.iter().map(move |action| {
                Entry::new(
                    Command::Run {
                        resource: resource.id,
                        action: action.id,
                    },
                    format!("{}: {}", resource.plural, action.title),
                    format!("opens {} first", resource.plural.to_lowercase()),
                    &action.keywords.join(" "),
                )
                .gated(&profile.evaluate(&action.requirement))
            })
        });

    let realm_entries = realms.iter().map(|realm| {
        Entry::new(
            Command::Realm(realm.clone()),
            format!("Realm: {realm}"),
            "Switch to this realm".to_owned(),
            "realm switch tenant",
        )
    });

    let globals = Global::ALL
        .iter()
        .map(|(global, title, subtitle, keywords)| {
            Entry::new(
                Command::Global(*global),
                (*title).to_owned(),
                (*subtitle).to_owned(),
                keywords,
            )
        });

    current_actions
        .chain(screens)
        .chain(globals)
        .chain(realm_entries)
        .chain(other_actions)
        .collect()
}

/// Fuzzy-rank entries against what the operator typed.
///
/// An empty needle keeps catalog order, which puts the current view's actions first.
#[must_use]
pub fn search(entries: &[Entry], needle: &str) -> Vec<usize> {
    if needle.trim().is_empty() {
        return (0..entries.len()).collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(needle, CaseMatching::Ignore, Normalization::Smart);

    let mut scored: Vec<(usize, u32)> = entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let mut buffer = Vec::new();
            pattern
                .score(Utf32Str::new(&entry.haystack, &mut buffer), &mut matcher)
                .map(|score| (index, score))
        })
        .collect();

    // Ties keep catalog order, so results never shuffle between keystrokes.
    scored.sort_by(|(left_index, left), (right_index, right)| {
        right.cmp(left).then(left_index.cmp(right_index))
    });

    scored.into_iter().map(|(index, _)| index).collect()
}

#[cfg(test)]
mod tests;
