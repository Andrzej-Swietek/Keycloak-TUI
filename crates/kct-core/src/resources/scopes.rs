use super::prelude::*;

pub(crate) const CLIENT_SCOPES: Resource = Resource {
    id: "client-scopes",
    singular: "Client scope",
    plural: "Client scopes",
    section: Section::Configure,
    collection: ep::CLIENT_SCOPES,
    item: Some(ep::CLIENT_SCOPE),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::client_scope),
    label: &["/name", "/id"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
        Column::new("Protocol", Projection::At("/protocol"), Width::Fixed(14)),
        Column::new(
            "In token scope",
            Projection::At("/attributes/include.in.token.scope"),
            Width::Fixed(15),
        ),
        Column::new(
            "Consent screen",
            Projection::At("/attributes/display.on.consent.screen"),
            Width::Fixed(15),
        )
        .toned(Tone::Muted),
        Column::new("Description", Projection::At("/description"), Width::Flex(4))
            .toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Details",
            fields: &[
                Field::new("ID", "/id", FieldKind::ReadOnly),
                Field::new("Name", "/name", FieldKind::Text),
                Field::new("Description", "/description", FieldKind::Multiline),
                Field::new("Protocol", "/protocol", FieldKind::Choice(&["openid-connect", "saml"])),
                Field::stringly("Include in token scope", "/attributes/include.in.token.scope", FieldKind::Bool)
                    .help("Off means the scope's mappers run but the scope never appears in the `scope` claim."),
                Field::stringly("Display on consent screen", "/attributes/display.on.consent.screen", FieldKind::Bool),
                Field::stringly("Consent screen text", "/attributes/consent.screen.text", FieldKind::Text),
                Field::stringly("GUI order", "/attributes/gui.order", FieldKind::Text),
            ],
        },
        Tab::Nested {
            title: "Mappers",
            resource: "scope-mappers",
            slot: "scope",
            source: None,
            preset: &[],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new("create", "Create client scope", Verb::Create, Danger::Confirm),
        Action::new("edit", "Edit client scope", Verb::Edit, Danger::Confirm),
        Action::new("delete", "Delete client scope", Verb::Delete, Danger::Destructive),
    ],
    keywords: &["scope", "claims", "consent", "openid", "profile", "email"],
    summary: "Reusable bundles of protocol mappers and role scope.",
};

pub(crate) const SCOPE_MAPPERS: Resource = Resource {
    id: "scope-mappers",
    singular: "Protocol mapper",
    plural: "Protocol mappers",
    section: Section::Configure,
    collection: ep::CLIENT_SCOPE_PROTOCOL_MAPPERS,
    item: Some(ep::CLIENT_SCOPE_PROTOCOL_MAPPER),
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
    keywords: &["claim", "token"],
    summary: "Mappers carried by this client scope.",
};
