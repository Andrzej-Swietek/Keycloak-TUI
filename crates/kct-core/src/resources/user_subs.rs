use super::prelude::*;

pub(crate) const USER_CREDENTIALS: Resource = Resource {
    id: "user-credentials",
    singular: "Credential",
    plural: "Credentials",
    section: Section::Manage,
    collection: ep::USER_CREDENTIALS,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: None,
    label: &["/type", "/userLabel", "/id"],
    columns: &[
        Column::new("Type", Projection::At("/type"), Width::Flex(2)).toned(Tone::Primary),
        Column::new("Label", Projection::At("/userLabel"), Width::Flex(3)),
        Column::new(
            "Created",
            Projection::EpochMillis("/createdDate"),
            Width::Fixed(19),
        ),
        Column::new("Data", Projection::At("/credentialData"), Width::Flex(4)).toned(Tone::Muted),
    ],
    search: Search::None,
    ops: Ops::RO.with(Ops::DELETE),
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[Action::new(
        "delete",
        "Delete credential",
        Verb::Delete,
        Danger::Destructive,
    )
    .keywords(&["remove otp", "reset mfa"])],
    keywords: &["password", "otp", "webauthn", "passkey", "mfa"],
    summary: "Credentials registered for this user.",
};

pub(crate) const USER_GROUPS: Resource = Resource {
    id: "user-groups",
    singular: "Group",
    plural: "Groups",
    section: Section::Manage,
    collection: ep::USER_GROUPS,
    item: Some(ep::GROUP),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: None,
    label: &["/path", "/name"],
    columns: GROUP_COLUMNS,
    search: Search::Client,
    ops: Ops::RO.with(Ops::DELETE),
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[
        Action::new(
            "join",
            "Join group",
            Verb::Special("join-group"),
            Danger::Confirm,
        ),
        Action::new("delete", "Leave group", Verb::Delete, Danger::Confirm),
    ],
    keywords: &["membership", "team"],
    summary: "Groups this user belongs to.",
};

pub(crate) const USER_SESSIONS: Resource = Resource {
    id: "user-sessions",
    singular: "Session",
    plural: "Sessions",
    section: Section::Observe,
    collection: ep::USER_SESSIONS,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: None,
    label: &["/id"],
    columns: &[
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
        Column::new(
            "Clients",
            Projection::Compute(session_clients),
            Width::Flex(4),
        ),
        Column::new("ID", Projection::At("/id"), Width::Flex(3)).toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[Action::new(
        "revoke",
        "Revoke session",
        Verb::Special("revoke-session"),
        Danger::Destructive,
    )],
    keywords: &["logged in", "sso", "revoke"],
    summary: "Active sessions for this user.",
};

pub(crate) fn session_clients(row: &Value) -> String {
    row.get("clients")
        .and_then(Value::as_object)
        .map(|clients| {
            clients
                .values()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}
