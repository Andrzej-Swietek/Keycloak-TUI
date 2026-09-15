use super::prelude::*;

pub(crate) const GROUPS: Resource = Resource {
    id: "groups",
    singular: "Group",
    plural: "Groups",
    section: Section::Manage,
    collection: ep::GROUPS,
    item: Some(ep::GROUP),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::group),
    label: &["/path", "/name", "/id"],
    columns: GROUP_COLUMNS,
    search: Search::ServerWithAttributes,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Details",
            fields: GROUP_FIELDS,
        },
        Tab::Nested {
            title: "Child groups",
            resource: "group-children",
            slot: "group",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Members",
            resource: "group-members",
            slot: "group",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Role mapping",
            resource: "role-mappings",
            slot: "owner",
            source: None,
            preset: &[("ownerKind", "groups")],
        },
        Tab::Nested {
            title: "Client roles",
            resource: "client-role-mappings",
            slot: "owner",
            source: None,
            preset: &[("ownerKind", "groups")],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new("create", "Create group", Verb::Create, Danger::Confirm),
        Action::new(
            "assign-role",
            "Assign realm role",
            Verb::Special("assign-role"),
            Danger::Confirm,
        )
        .keywords(&["grant", "add role", "give permission", "rbac"]),
        Action::new(
            "unassign-role",
            "Remove realm role",
            Verb::Special("unassign-role"),
            Danger::Destructive,
        )
        .keywords(&["revoke", "take away role"]),
        Action::new(
            "assign-client-role",
            "Assign client role",
            Verb::Special("pick-client-for-role"),
            Danger::Confirm,
        )
        .keywords(&["client role", "grant", "application role", "rbac"]),
        Action::new("edit", "Edit group", Verb::Edit, Danger::Confirm),
        Action::new("delete", "Delete group", Verb::Delete, Danger::Destructive)
            .keywords(&["remove", "drop subtree"]),
    ],
    keywords: &["ou", "organizational unit", "team", "membership"],
    summary: "Hierarchical groups; role mappings and attributes cascade to members.",
};

pub(crate) const GROUP_CHILDREN: Resource = Resource {
    id: "group-children",
    singular: "Child group",
    plural: "Child groups",
    section: Section::Manage,
    collection: ep::GROUP_CHILDREN,
    item: Some(ep::GROUP),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::group),
    label: &["/name", "/path", "/id"],
    columns: GROUP_COLUMNS,
    search: Search::Server,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Details",
            fields: GROUP_FIELDS,
        },
        Tab::Nested {
            title: "Members",
            resource: "group-members",
            slot: "group",
            source: None,
            preset: &[],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new(
            "create",
            "Create child group",
            Verb::Create,
            Danger::Confirm,
        ),
        Action::new("delete", "Delete group", Verb::Delete, Danger::Destructive),
    ],
    keywords: &["subgroup", "nested group"],
    summary: "Groups nested directly under the selected group.",
};
