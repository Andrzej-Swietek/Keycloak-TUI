use super::prelude::*;

const SCOPE_COLUMNS: &[Column] = &[
    Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
    Column::new("ID", Projection::At("/id"), Width::Flex(3)).toned(Tone::Muted),
];

const SCOPE_ACTIONS: &[Action] = &[
    Action::new(
        "add",
        "Add client scope",
        Verb::Special("add-client-scope"),
        Danger::Confirm,
    ),
    Action::new(
        "delete",
        "Remove client scope",
        Verb::Delete,
        Danger::Confirm,
    ),
];

pub(crate) const CLIENT_DEFAULT_SCOPES: Resource = Resource {
    id: "client-default-scopes",
    singular: "Default scope",
    plural: "Default client scopes",
    section: Section::Configure,
    collection: ep::CLIENT_DEFAULT_SCOPES,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: None,
    label: &["/name"],
    columns: SCOPE_COLUMNS,
    search: Search::Client,
    ops: Ops::RO.with(Ops::DELETE),
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: SCOPE_ACTIONS,
    keywords: &["always applied", "scope"],
    summary: "Scopes applied to every token this client obtains.",
};

pub(crate) const CLIENT_OPTIONAL_SCOPES: Resource = Resource {
    id: "client-optional-scopes",
    singular: "Optional scope",
    plural: "Optional client scopes",
    section: Section::Configure,
    collection: ep::CLIENT_OPTIONAL_SCOPES,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: None,
    label: &["/name"],
    columns: SCOPE_COLUMNS,
    search: Search::Client,
    ops: Ops::RO.with(Ops::DELETE),
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: SCOPE_ACTIONS,
    keywords: &["requested scope", "on demand"],
    summary: "Scopes applied only when the client asks for them.",
};

/// The endpoint requires `scope`; empty means the client's default scopes.
pub(crate) const CLIENT_EFFECTIVE_MAPPERS: Resource = Resource {
    id: "client-effective-mappers",
    singular: "Effective mapper",
    plural: "Effective protocol mappers",
    section: Section::Configure,
    collection: ep::CLIENT_EVALUATE_SCOPES_MAPPERS,
    item: None,
    shape: Shape::Array,
    fixed_params: &[("scope", "")],
    key: Key::custom("mapperId"),
    new_object: None,
    label: &["/mapperName"],
    columns: &[
        Column::new("Mapper", Projection::At("/mapperName"), Width::Flex(3)).toned(Tone::Primary),
        Column::new(
            "From scope",
            Projection::At("/containerName"),
            Width::Flex(3),
        ),
        Column::new(
            "Container",
            Projection::At("/containerType"),
            Width::Fixed(14),
        )
        .toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[],
    keywords: &["why is my claim missing", "evaluate", "effective claims"],
    summary: "Every mapper that will run for this client, and which scope contributed it.",
};
