use super::prelude::*;

pub(crate) const AUTH_FLOWS: Resource = Resource {
    id: "auth-flows",
    singular: "Authentication flow",
    plural: "Authentication flows",
    section: Section::Configure,
    collection: ep::AUTH_FLOWS,
    item: Some(ep::AUTH_FLOW),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::auth_flow),
    label: &["/alias", "/id"],
    columns: &[
        Column::new("Alias", Projection::At("/alias"), Width::Flex(3)).toned(Tone::Primary),
        Column::new(
            "Description",
            Projection::At("/description"),
            Width::Flex(5),
        )
        .toned(Tone::Muted),
        Column::new(
            "Built-in",
            Projection::Bool {
                pointer: "/builtIn",
                yes: "yes",
                no: "",
            },
            Width::Fixed(9),
        )
        .toned(Tone::Muted),
        Column::new(
            "Top level",
            Projection::Bool {
                pointer: "/topLevel",
                yes: "yes",
                no: "",
            },
            Width::Fixed(10),
        ),
        Column::new("Type", Projection::At("/providerId"), Width::Fixed(14)),
    ],
    search: Search::Client,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Nested {
            title: "Executions",
            resource: "flow-executions",
            slot: "flow",
            source: Some("/alias"),
            preset: &[],
        },
        Tab::Form {
            title: "Details",
            fields: &[
                Field::new("ID", "/id", FieldKind::ReadOnly),
                Field::new("Alias", "/alias", FieldKind::Text),
                Field::new("Description", "/description", FieldKind::Multiline),
                Field::new("Built-in", "/builtIn", FieldKind::ReadOnly)
                    .help("Built-in flows cannot be edited; duplicate one first."),
                Field::new("Top level", "/topLevel", FieldKind::ReadOnly),
            ],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new("create", "Create flow", Verb::Create, Danger::Confirm),
        Action::new(
            "duplicate",
            "Duplicate flow",
            Verb::Special("copy-flow"),
            Danger::Confirm,
        )
        .keywords(&["copy", "clone", "customise built-in"]),
        Action::new("delete", "Delete flow", Verb::Delete, Danger::Destructive),
        Action::new(
            "bind",
            "Bind flow to realm",
            Verb::Special("bind-flow"),
            Danger::Destructive,
        )
        .keywords(&[
            "browser flow",
            "direct grant",
            "reset credentials",
            "make default",
        ]),
    ],
    keywords: &[
        "login flow",
        "mfa",
        "otp",
        "2fa",
        "webauthn",
        "conditional",
        "step-up",
        "acr",
    ],
    summary: "Browser, direct-grant and broker flows, with their executions.",
};
