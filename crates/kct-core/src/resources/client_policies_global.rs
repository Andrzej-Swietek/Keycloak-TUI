//! The client profiles and policies shipped with the server.
//!
//! Both arrive in the same document as the realm's own, behind an `include-global-*` query
//! parameter. Keycloak answers 400 to a `PUT` that changes one, so these screens are
//! read-only; a policy references a built-in profile by name.

use super::prelude::*;

pub(crate) const CLIENT_PROFILES_GLOBAL: Resource = Resource {
    id: "client-profiles-global",
    singular: "Built-in profile",
    plural: "Built-in profiles",
    section: Section::Realm,
    collection: ep::CLIENT_POLICIES_PROFILES,
    item: None,
    shape: Shape::ArrayAt("/globalProfiles"),
    fixed_params: &[("include-global-profiles", "true")],
    key: Key::NAME,
    new_object: None,
    label: &["/name"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(2)).toned(Tone::Primary),
        Column::new(
            "Executors",
            Projection::Count("/executors"),
            Width::Fixed(10),
        ),
        Column::new(
            "Description",
            Projection::At("/description"),
            Width::Flex(5),
        )
        .toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Feature(features::CLIENT_POLICIES),
    tabs: &[Tab::Raw],
    actions: &[],
    keywords: &[
        "fapi",
        "oauth 2.1",
        "global profile",
        "conformance",
        "read only",
    ],
    summary: "Profiles shipped with the server. Reference them by name; they cannot be changed.",
};

pub(crate) const CLIENT_POLICIES_GLOBAL: Resource = Resource {
    id: "client-policies-global",
    singular: "Built-in policy",
    plural: "Built-in policies",
    section: Section::Realm,
    collection: ep::CLIENT_POLICIES_POLICIES,
    item: None,
    shape: Shape::ArrayAt("/globalPolicies"),
    fixed_params: &[("include-global-policies", "true")],
    key: Key::NAME,
    new_object: None,
    label: &["/name"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(2)).toned(Tone::Primary),
        Column::new(
            "Profiles",
            Projection::Join {
                pointer: "/profiles",
                separator: ", ",
            },
            Width::Flex(3),
        ),
        Column::new(
            "Description",
            Projection::At("/description"),
            Width::Flex(4),
        )
        .toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Feature(features::CLIENT_POLICIES),
    tabs: &[Tab::Raw],
    actions: &[],
    keywords: &["global policy", "conformance", "read only"],
    summary: "Policies shipped with the server. They cannot be changed.",
};
