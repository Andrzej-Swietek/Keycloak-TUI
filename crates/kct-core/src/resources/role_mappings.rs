use super::prelude::*;

pub(crate) const ROLE_MAPPINGS: Resource = Resource {
    id: "role-mappings",
    singular: "Role mapping",
    plural: "Realm role mappings",
    section: Section::Manage,
    collection: ep::ROLE_MAPPINGS_REALM,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: None,
    label: &["/name"],
    columns: ROLE_COLUMNS,
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[
        Action::new(
            "assign",
            "Assign realm role",
            Verb::Special("assign-role"),
            Danger::Confirm,
        )
        .keywords(&["grant", "add role"]),
        Action::new(
            "unassign",
            "Remove realm role",
            Verb::Special("unassign-role"),
            Danger::Destructive,
        )
        .keywords(&["revoke", "take away"]),
    ],
    keywords: &["grant", "authority", "rbac"],
    summary: "Realm roles assigned directly to this user or group.",
};

/// The composite endpoint answers with `{realmMappings, clientMappings}`; the per-client half
/// is an object keyed by client id, which reads naturally as one row per client.
pub(crate) const CLIENT_ROLE_MAPPINGS: Resource = Resource {
    id: "client-role-mappings",
    singular: "Client role mapping",
    plural: "Client role mappings",
    section: Section::Manage,
    collection: ep::ROLE_MAPPINGS,
    item: None,
    shape: Shape::EntriesAt("/clientMappings"),
    fixed_params: &[],
    key: Key::custom("client"),
    new_object: None,
    label: &["/client"],
    columns: &[
        Column::new("Client", Projection::At("/client"), Width::Flex(3)).toned(Tone::Primary),
        Column::new("Roles", Projection::Count("/mappings"), Width::Fixed(8)),
        Column::new(
            "Assigned",
            Projection::Compute(mapping_names),
            Width::Flex(8),
        ),
    ],
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[],
    keywords: &["client role", "grant"],
    summary: "Client roles assigned to this user or group, grouped by client.",
};

pub(crate) fn mapping_names(row: &Value) -> String {
    row.get("mappings")
        .and_then(Value::as_array)
        .map(|roles| {
            roles
                .iter()
                .filter_map(|role| role.get("name").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}
