use super::prelude::*;

pub(crate) const REQUIRED_ACTIONS: Resource = Resource {
    id: "required-actions",
    singular: "Required action",
    plural: "Required actions",
    section: Section::Configure,
    collection: ep::AUTH_REQUIRED_ACTIONS,
    item: Some(ep::AUTH_REQUIRED_ACTION),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ALIAS,
    new_object: None,
    label: &["/name", "/alias"],
    columns: &[
        Column::new("Alias", Projection::At("/alias"), Width::Flex(3)).toned(Tone::Primary),
        Column::new("Name", Projection::At("/name"), Width::Flex(4)),
        Column::new(
            "Enabled",
            Projection::Bool {
                pointer: "/enabled",
                yes: "yes",
                no: "no",
            },
            Width::Fixed(8),
        )
        .toned(Tone::Flag),
        Column::new(
            "Default",
            Projection::Bool {
                pointer: "/defaultAction",
                yes: "yes",
                no: "",
            },
            Width::Fixed(9),
        ),
        Column::new("Priority", Projection::At("/priority"), Width::Fixed(9)).toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::RW,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Details",
            fields: &[
                Field::new("Alias", "/alias", FieldKind::ReadOnly),
                Field::new("Name", "/name", FieldKind::Text),
                Field::new("Enabled", "/enabled", FieldKind::Bool),
                Field::new("Default action", "/defaultAction", FieldKind::Bool)
                    .help("Applied to every newly created user."),
                Field::new("Priority", "/priority", FieldKind::Integer),
                Field::new("Config", "/config", FieldKind::StringMap),
            ],
        },
        Tab::Raw,
    ],
    actions: &[Action::new(
        "edit",
        "Edit required action",
        Verb::Edit,
        Danger::Confirm,
    )],
    keywords: &[
        "verify email",
        "update password",
        "configure otp",
        "terms and conditions",
    ],
    summary: "Actions Keycloak can force a user to complete at login.",
};

pub(crate) const CLIENT_INITIAL_ACCESS: Resource = Resource {
    id: "client-initial-access",
    singular: "Initial access token",
    plural: "Initial access tokens",
    section: Section::Configure,
    collection: ep::CLIENT_INITIAL_ACCESS,
    item: Some(ep::CLIENT_INITIAL_ACCESS_ITEM),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::initial_access_token),
    label: &["/id"],
    columns: &[
        Column::new("ID", Projection::At("/id"), Width::Flex(3)).toned(Tone::Primary),
        Column::new(
            "Created",
            Projection::EpochMillis("/timestamp"),
            Width::Fixed(19),
        ),
        Column::new(
            "Expiration (s)",
            Projection::At("/expiration"),
            Width::Fixed(14),
        ),
        Column::new("Count", Projection::At("/count"), Width::Fixed(7)),
        Column::new(
            "Remaining",
            Projection::At("/remainingCount"),
            Width::Fixed(10),
        ),
    ],
    search: Search::None,
    ops: Ops::LIST.with(Ops::CREATE).with(Ops::DELETE),
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[
        Action::new(
            "create",
            "Create initial access token",
            Verb::Create,
            Danger::Confirm,
        )
        .keywords(&["dynamic client registration", "bootstrap"]),
        Action::new("delete", "Revoke token", Verb::Delete, Danger::Destructive),
    ],
    keywords: &["dynamic client registration", "kcreg", "provisioning"],
    summary: "Tokens that allow self-service client registration.",
};
