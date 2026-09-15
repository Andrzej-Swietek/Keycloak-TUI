//! Client policies and the profiles they enforce.
//!
//! Both endpoints answer one document holding an array. Adding
//! `include-global-profiles`/`include-global-policies` adds a second array of built-ins:
//! Keycloak answers 400 to a `PUT` that changes one of those, so the editable screens read
//! the document without the flag and can only ever write the realm's own entries.

use super::prelude::*;
use crate::path::PathParams;

pub(crate) const CLIENT_PROFILE_ITEMS: Resource = Resource {
    id: "client-profile-items",
    singular: "Client profile",
    plural: "Client profiles",
    section: Section::Realm,
    collection: ep::CLIENT_POLICIES_PROFILES,
    item: None,
    shape: Shape::ArrayAt("/profiles"),
    fixed_params: &[],
    key: Key::NAME,
    new_object: Some(new_profile),
    label: &["/name"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(2)).toned(Tone::Primary),
        Column::new("Executors", Projection::Count("/executors"), Width::Fixed(10)),
        Column::new("Description", Projection::At("/description"), Width::Flex(5))
            .toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::CRUD,
    requirement: Requirement::Feature(features::CLIENT_POLICIES),
    tabs: &[
        Tab::Form {
            title: "Profile",
            fields: &[
                Field::new("Name", "/name", FieldKind::Text)
                    .help("Referenced by a policy's Profiles list."),
                Field::new("Description", "/description", FieldKind::Multiline),
                Field::new("Executors", "/executors", FieldKind::Json).help(
                    "`[{\"executor\": \"secure-session\", \"configuration\": {}}]` — see the \
                     Global profiles screen for worked examples.",
                ),
            ],
        },
        Tab::Raw,
    ],
    actions: &[Action::new(
        "delete",
        "Delete client profile",
        Verb::Delete,
        Danger::Destructive,
    )],
    keywords: &["fapi", "executor", "conformance", "client profile"],
    summary: "Executor bundles this realm defines, referenced by client policies.",
};

pub(crate) const CLIENT_POLICY_ITEMS: Resource = Resource {
    id: "client-policy-items",
    singular: "Client policy",
    plural: "Client policies",
    section: Section::Realm,
    collection: ep::CLIENT_POLICIES_POLICIES,
    item: None,
    shape: Shape::ArrayAt("/policies"),
    fixed_params: &[],
    key: Key::NAME,
    new_object: Some(new_policy),
    label: &["/name"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(2)).toned(Tone::Primary),
        Column::new(
            "Enabled",
            Projection::Bool {
                pointer: "/enabled",
                yes: "yes",
                no: "DISABLED",
            },
            Width::Fixed(9),
        )
        .toned(Tone::Flag),
        Column::new(
            "Profiles",
            Projection::Join {
                pointer: "/profiles",
                separator: ", ",
            },
            Width::Flex(3),
        ),
        Column::new("Conditions", Projection::Count("/conditions"), Width::Fixed(11)),
        Column::new("Description", Projection::At("/description"), Width::Flex(3))
            .toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::CRUD,
    requirement: Requirement::Feature(features::CLIENT_POLICIES),
    tabs: &[
        Tab::Form {
            title: "Policy",
            fields: &[
                Field::new("Name", "/name", FieldKind::Text),
                Field::new("Description", "/description", FieldKind::Multiline),
                Field::new("Enabled", "/enabled", FieldKind::Bool),
                Field::new("Profiles", "/profiles", FieldKind::List)
                    .help("Names of the profiles this policy enforces, global ones included."),
                Field::new("Conditions", "/conditions", FieldKind::Json).help(
                    "`[{\"condition\": \"client-roles\", \"configuration\": {}}]` — a client \
                     must match all of them for the profiles to apply.",
                ),
            ],
        },
        Tab::Raw,
    ],
    actions: &[Action::new(
        "delete",
        "Delete client policy",
        Verb::Delete,
        Danger::Destructive,
    )],
    keywords: &["fapi", "conformance", "condition", "client policy"],
    summary: "Conditions this realm enforces on clients, and the profiles they apply.",
};

fn new_profile(name: &str, _params: &PathParams) -> Value {
    serde_json::json!({"name": name, "description": "", "executors": []})
}

fn new_policy(name: &str, _params: &PathParams) -> Value {
    serde_json::json!({
        "name": name,
        "description": "",
        "enabled": false,
        "conditions": [],
        "profiles": [],
    })
}
