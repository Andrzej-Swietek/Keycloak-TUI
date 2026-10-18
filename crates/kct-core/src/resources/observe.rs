use super::prelude::*;

pub(crate) const CLIENT_SESSION_STATS: Resource = Resource {
    id: "sessions",
    singular: "Client sessions",
    plural: "Sessions",
    section: Section::Observe,
    collection: ep::SESSIONS_BY_CLIENT,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: None,
    label: &["/clientId", "/id"],
    columns: &[
        Column::new("Client", Projection::At("/clientId"), Width::Flex(4)).toned(Tone::Primary),
        Column::new("Active", Projection::At("/active"), Width::Fixed(10)),
        Column::new("Offline", Projection::At("/offline"), Width::Fixed(10)).toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[
        Action::new(
            "logout-all",
            "Log out every session in realm",
            Verb::Post("logout-all"),
            Danger::Destructive,
        )
        .keywords(&["kill all", "revoke all", "sign everyone out"]),
        Action::new(
            "push-revocation",
            "Push revocation policy",
            Verb::Post("push-revocation"),
            Danger::Confirm,
        ),
    ],
    keywords: &[
        "active sessions",
        "offline",
        "who is logged in",
        "sso session",
    ],
    summary: "Per-client counts of active and offline sessions.",
};

pub(crate) const LOGIN_EVENTS: Resource = Resource {
    id: "events",
    singular: "Login event",
    plural: "Login events",
    section: Section::Observe,
    collection: ep::EVENTS,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::custom("time"),
    new_object: None,
    label: &["/type"],
    columns: &[
        Column::new("Time", Projection::EpochMillis("/time"), Width::Fixed(19)),
        Column::new("Type", Projection::At("/type"), Width::Flex(3)).toned(Tone::Primary),
        Column::new("Client", Projection::At("/clientId"), Width::Flex(3)),
        Column::new("User", Projection::Compute(event_user), Width::Flex(3)).toned(Tone::Muted),
        Column::new("IP", Projection::At("/ipAddress"), Width::Fixed(16)).toned(Tone::Muted),
        Column::new("Error", Projection::At("/error"), Width::Flex(3)).toned(Tone::Bad),
    ],
    search: Search::None,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[Action::new(
        "clear",
        "Clear stored login events",
        Verb::Delete,
        Danger::Destructive,
    )],
    keywords: &[
        "audit",
        "login failure",
        "brute force",
        "history",
        "who logged in",
    ],
    summary: "Recorded login, logout and token events. Requires user events to be enabled.",
};

pub(crate) const ADMIN_EVENTS: Resource = Resource {
    id: "admin-events",
    singular: "Admin event",
    plural: "Admin events",
    section: Section::Observe,
    collection: ep::ADMIN_EVENTS,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::custom("time"),
    new_object: None,
    label: &["/resourcePath"],
    columns: &[
        Column::new("Time", Projection::EpochMillis("/time"), Width::Fixed(19)),
        Column::new(
            "Operation",
            Projection::At("/operationType"),
            Width::Fixed(10),
        )
        .toned(Tone::Primary),
        Column::new("Resource", Projection::At("/resourceType"), Width::Flex(2)),
        Column::new("Path", Projection::At("/resourcePath"), Width::Flex(5)),
        Column::new("By", Projection::At("/authDetails/userId"), Width::Flex(3)).toned(Tone::Muted),
        Column::new(
            "IP",
            Projection::At("/authDetails/ipAddress"),
            Width::Fixed(16),
        )
        .toned(Tone::Muted),
    ],
    search: Search::None,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[Action::new(
        "clear",
        "Clear stored admin events",
        Verb::Delete,
        Danger::Destructive,
    )],
    keywords: &[
        "audit",
        "who changed",
        "change log",
        "representation",
        "operation",
    ],
    summary: "Every configuration change, with the representation when details are on.",
};

/// The acting user, and who was behind them when an admin impersonated (Keycloak 26.8+).
fn event_user(row: &Value) -> String {
    let user = row
        .get("userId")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match row.pointer("/details/impersonator").and_then(Value::as_str) {
        Some(admin) => format!("{user} ← {admin}"),
        None => user.to_owned(),
    }
}
