use super::prelude::*;

pub(crate) const CLIENT_ROLES: Resource = Resource {
    id: "client-roles",
    singular: "Client role",
    plural: "Client roles",
    section: Section::Configure,
    collection: ep::CLIENT_ROLES,
    item: Some(ep::ROLE_BY_ID),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::NAME,
    new_object: Some(new_object::named),
    label: &["/name"],
    columns: ROLE_COLUMNS,
    search: Search::Server,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Details",
            fields: ROLE_FIELDS,
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new(
            "create",
            "Create client role",
            Verb::Create,
            Danger::Confirm,
        ),
        Action::new(
            "add-composite",
            "Add associated roles",
            Verb::Special("add-composite"),
            Danger::Confirm,
        )
        .keywords(&["composite", "nested role", "inherit", "grants transitively"]),
        Action::new("delete", "Delete role", Verb::Delete, Danger::Destructive),
    ],
    keywords: &["permission", "authority", "scope"],
    summary: "Roles defined by this client.",
};

pub(crate) const CLIENT_MAPPERS: Resource = Resource {
    id: "client-mappers",
    singular: "Protocol mapper",
    plural: "Protocol mappers",
    section: Section::Configure,
    collection: ep::CLIENT_PROTOCOL_MAPPERS,
    item: Some(ep::CLIENT_PROTOCOL_MAPPER),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::protocol_mapper),
    label: &["/name"],
    columns: MAPPER_COLUMNS,
    search: Search::Client,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Details",
            fields: MAPPER_FIELDS,
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new("create", "Add mapper", Verb::Create, Danger::Confirm),
        Action::new("delete", "Delete mapper", Verb::Delete, Danger::Confirm),
    ],
    keywords: &["claim", "token", "attribute mapper", "audience"],
    summary: "Mappers that put claims into this client's tokens.",
};

pub(crate) const CLIENT_SESSIONS: Resource = Resource {
    id: "client-sessions",
    singular: "Session",
    plural: "Sessions",
    section: Section::Observe,
    collection: ep::CLIENT_SESSIONS,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: None,
    label: &["/id"],
    columns: &[
        Column::new("User", Projection::At("/username"), Width::Flex(3)).toned(Tone::Primary),
        Column::new(
            "Started",
            Projection::EpochMillis("/start"),
            Width::Fixed(19),
        ),
        Column::new(
            "Last access",
            Projection::EpochMillis("/lastAccess"),
            Width::Fixed(19),
        ),
        Column::new("IP", Projection::At("/ipAddress"), Width::Fixed(16)),
    ],
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[],
    keywords: &["who is using", "active"],
    summary: "Users currently holding a session with this client.",
};
