use super::prelude::*;

pub(crate) const CLIENT_AUTHZ_RESOURCES: Resource = Resource {
    id: "client-authz-resources",
    singular: "Authz resource",
    plural: "Authorization resources",
    section: Section::Configure,
    collection: ep::CLIENT_AUTHZ_RESOURCES,
    item: Some(ep::CLIENT_AUTHZ_RESOURCE),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::custom("_id"),
    new_object: Some(new_object::named),
    label: &["/name"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
        Column::new("Type", Projection::At("/type"), Width::Flex(3)),
        Column::new(
            "URIs",
            Projection::Join {
                pointer: "/uris",
                separator: ", ",
            },
            Width::Flex(4),
        ),
        Column::new(
            "Owner-managed",
            Projection::Bool {
                pointer: "/ownerManagedAccess",
                yes: "yes",
                no: "",
            },
            Width::Fixed(14),
        ),
    ],
    search: Search::Server,
    ops: Ops::CRUD,
    requirement: Requirement::Feature(features::AUTHORIZATION),
    tabs: &[Tab::Raw],
    actions: &[
        Action::new("create", "Create resource", Verb::Create, Danger::Confirm),
        Action::new(
            "delete",
            "Delete resource",
            Verb::Delete,
            Danger::Destructive,
        ),
    ],
    keywords: &["uma", "authz", "protected resource"],
    summary: "Protected resources of this resource server.",
};

pub(crate) const CLIENT_AUTHZ_SCOPES: Resource = Resource {
    id: "client-authz-scopes",
    singular: "Authz scope",
    plural: "Authorization scopes",
    section: Section::Configure,
    collection: ep::CLIENT_AUTHZ_SCOPES,
    item: Some(ep::CLIENT_AUTHZ_SCOPE),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::named),
    label: &["/name"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
        Column::new(
            "Display name",
            Projection::At("/displayName"),
            Width::Flex(3),
        ),
    ],
    search: Search::Server,
    ops: Ops::CRUD,
    requirement: Requirement::Feature(features::AUTHORIZATION),
    tabs: &[Tab::Raw],
    actions: &[
        Action::new("create", "Create scope", Verb::Create, Danger::Confirm),
        Action::new("delete", "Delete scope", Verb::Delete, Danger::Destructive),
    ],
    keywords: &["uma", "authz"],
    summary: "Scopes that can be requested against this resource server.",
};

pub(crate) const CLIENT_AUTHZ_POLICIES: Resource = Resource {
    id: "client-authz-policies",
    singular: "Authz policy",
    plural: "Authorization policies",
    section: Section::Configure,
    collection: ep::CLIENT_AUTHZ_POLICIES,
    item: Some(ep::CLIENT_AUTHZ_POLICY),
    shape: Shape::Array,
    fixed_params: &[("permission", "false")],
    key: Key::ID,
    new_object: Some(new_object::named),
    label: &["/name"],
    columns: AUTHZ_NAME_COLUMNS,
    search: Search::Server,
    ops: Ops::CRUD,
    requirement: Requirement::Feature(features::AUTHORIZATION),
    tabs: &[Tab::Raw],
    actions: &[
        Action::new(
            "create",
            "Create policy",
            Verb::Special("new-authz-policy"),
            Danger::Confirm,
        )
        .keywords(&["policy type", "spi", "provider"]),
        Action::new("delete", "Delete policy", Verb::Delete, Danger::Destructive),
    ],
    keywords: &[
        "rbac",
        "abac",
        "js policy",
        "group policy",
        "role policy",
        "time policy",
    ],
    summary: "Policies evaluated when this resource server authorizes a request.",
};

pub(crate) const CLIENT_AUTHZ_PERMISSIONS: Resource = Resource {
    id: "client-authz-permissions",
    singular: "Authz permission",
    plural: "Authorization permissions",
    section: Section::Configure,
    collection: ep::CLIENT_AUTHZ_PERMISSIONS,
    item: Some(ep::CLIENT_AUTHZ_PERMISSION),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::named),
    label: &["/name"],
    columns: AUTHZ_NAME_COLUMNS,
    search: Search::Server,
    ops: Ops::CRUD,
    requirement: Requirement::Feature(features::AUTHORIZATION),
    tabs: &[Tab::Raw],
    actions: &[
        Action::new(
            "create",
            "Create permission",
            Verb::Special("new-authz-permission"),
            Danger::Confirm,
        )
        .keywords(&["permission type", "resource", "scope"]),
        Action::new(
            "delete",
            "Delete permission",
            Verb::Delete,
            Danger::Destructive,
        ),
    ],
    keywords: &[
        "resource permission",
        "scope permission",
        "fine-grained admin permissions",
        "fgap",
        "admin-permissions",
        "delegated admin",
    ],
    summary: "Bindings of policies to resources and scopes.",
};
