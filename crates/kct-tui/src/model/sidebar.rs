//! The navigation tree, projected from the catalog.

use kct_core::catalog::{Resource, Section};

/// Flattened navigation tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarRow {
    Heading(Section),
    Item(&'static Resource),
}

/// Every top-level screen, grouped by section.
///
/// Screens this server cannot serve are included; the caller greys them. Sections with no
/// screens are omitted.
#[must_use]
pub fn sidebar_rows() -> Vec<SidebarRow> {
    Section::ALL
        .iter()
        .flat_map(|section| {
            let screens: Vec<SidebarRow> = kct_core::resources::in_section(*section)
                .filter(|resource| !resource.is_nested())
                .map(SidebarRow::Item)
                .collect();

            if screens.is_empty() {
                Vec::new()
            } else {
                std::iter::once(SidebarRow::Heading(*section))
                    .chain(screens)
                    .collect()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sidebar_lists_only_screens_reachable_without_a_parent() {
        let rows = sidebar_rows();
        let items: Vec<&str> = rows
            .iter()
            .filter_map(|row| match row {
                SidebarRow::Item(resource) => Some(resource.id),
                SidebarRow::Heading(_) => None,
            })
            .collect();

        assert!(items.contains(&"users"));
        assert!(
            !items.contains(&"user-sessions"),
            "needs a user to make sense"
        );
        assert!(
            rows.iter()
                .any(|row| matches!(row, SidebarRow::Heading(Section::Manage)))
        );
        assert!(
            !rows
                .iter()
                .any(|row| matches!(row, SidebarRow::Heading(Section::Tools))),
            "a section with no screens is not worth a heading"
        );
    }
}
