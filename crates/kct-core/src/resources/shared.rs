use super::prelude::*;

pub(crate) const KC_26: Requirement = Requirement::MinVersion(Version::new(26, 0, 0));

pub(crate) const KC_26_2: Requirement = Requirement::MinVersion(Version::new(26, 2, 0));
pub(crate) const KC_26_8: Requirement = Requirement::MinVersion(Version::new(26, 8, 0));

pub(crate) const USER_COLUMNS: &[Column] = &[
    Column::new("Username", Projection::At("/username"), Width::Flex(3)).toned(Tone::Primary),
    Column::new("Email", Projection::At("/email"), Width::Flex(3)),
    Column::new("Name", Projection::Compute(full_name), Width::Flex(3)),
    Column::new(
        "Enabled",
        Projection::Bool {
            pointer: "/enabled",
            yes: "yes",
            no: "DISABLED",
        },
        Width::Fixed(8),
    )
    .toned(Tone::Flag),
    Column::new(
        "Verified",
        Projection::Bool {
            pointer: "/emailVerified",
            yes: "yes",
            no: "no",
        },
        Width::Fixed(8),
    )
    .toned(Tone::Muted),
    Column::new(
        "Federated",
        Projection::At("/federationLink"),
        Width::Fixed(12),
    )
    .toned(Tone::Muted),
    Column::new(
        "Created",
        Projection::EpochMillis("/createdTimestamp"),
        Width::Fixed(19),
    )
    .toned(Tone::Muted),
];

pub(crate) fn full_name(row: &Value) -> String {
    let first = row.get("firstName").and_then(Value::as_str).unwrap_or("");
    let last = row.get("lastName").and_then(Value::as_str).unwrap_or("");
    format!("{first} {last}").trim().to_owned()
}

pub(crate) const GROUP_COLUMNS: &[Column] = &[
    Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
    Column::new("Path", Projection::At("/path"), Width::Flex(5)).toned(Tone::Muted),
    Column::new(
        "Sub-groups",
        Projection::Compute(subgroup_count),
        Width::Fixed(11),
    ),
    Column::new(
        "Attributes",
        Projection::Count("/attributes"),
        Width::Fixed(11),
    )
    .toned(Tone::Muted),
];

pub(crate) fn subgroup_count(row: &Value) -> String {
    // Keycloak 23+ reports `subGroupCount`; older servers only embed `subGroups`.
    row.get("subGroupCount")
        .and_then(Value::as_u64)
        .or_else(|| {
            row.get("subGroups")
                .and_then(Value::as_array)
                .map(|s| s.len() as u64)
        })
        .map(|n| n.to_string())
        .unwrap_or_default()
}

pub(crate) const GROUP_FIELDS: &[Field] = &[
    Field::new("ID", "/id", FieldKind::ReadOnly),
    Field::new("Name", "/name", FieldKind::Text),
    Field::new("Path", "/path", FieldKind::ReadOnly)
        .help("Derived from the parent chain; change it by moving the group."),
    Field::new("Attributes", "/attributes", FieldKind::Attributes),
];

pub(crate) const ROLE_COLUMNS: &[Column] = &[
    Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
    Column::new(
        "Composite",
        Projection::Bool {
            pointer: "/composite",
            yes: "yes",
            no: "",
        },
        Width::Fixed(10),
    ),
    Column::new(
        "Description",
        Projection::At("/description"),
        Width::Flex(6),
    )
    .toned(Tone::Muted),
];

pub(crate) const ROLE_FIELDS: &[Field] = &[
    Field::new("ID", "/id", FieldKind::ReadOnly),
    Field::new("Name", "/name", FieldKind::Text),
    Field::new("Description", "/description", FieldKind::Multiline),
    Field::new("Composite", "/composite", FieldKind::ReadOnly)
        .help("Composite roles grant their associated roles transitively."),
    Field::new("Attributes", "/attributes", FieldKind::Attributes),
];

pub(crate) const MAPPER_COLUMNS: &[Column] = &[
    Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
    Column::new("Type", Projection::At("/protocolMapper"), Width::Flex(4)),
    Column::new(
        "Claim",
        Projection::At("/config/claim.name"),
        Width::Flex(3),
    ),
    Column::new(
        "Surfaces",
        Projection::Compute(mapper_surfaces),
        Width::Fixed(22),
    )
    .toned(Tone::Muted),
];

/// The four per-surface toggles. A mapper emits its claim only where the toggle is on.
pub(crate) fn mapper_surfaces(row: &Value) -> String {
    let on = |key: &str| {
        row.pointer(&format!("/config/{key}"))
            .is_some_and(|v| v == "true" || v == true)
    };
    [
        ("access", on("access.token.claim")),
        ("id", on("id.token.claim")),
        ("userinfo", on("userinfo.token.claim")),
        ("introspect", on("introspection.token.claim")),
    ]
    .into_iter()
    .filter_map(|(label, enabled)| enabled.then_some(label))
    .collect::<Vec<_>>()
    .join(",")
}

pub(crate) const MAPPER_FIELDS: &[Field] = &[
    Field::new("ID", "/id", FieldKind::ReadOnly),
    Field::new("Name", "/name", FieldKind::Text),
    Field::new("Mapper type", "/protocolMapper", FieldKind::Text),
    Field::new(
        "Protocol",
        "/protocol",
        FieldKind::Choice(&["openid-connect", "saml"]),
    ),
    Field::new("Consent required", "/consentRequired", FieldKind::Bool),
    Field::new("Config", "/config", FieldKind::StringMap)
        .help("Mapper-specific keys; the per-surface toggles live here too."),
];

pub(crate) const AUTHZ_NAME_COLUMNS: &[Column] = &[
    Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
    Column::new("Type", Projection::At("/type"), Width::Flex(2)),
    Column::new(
        "Decision strategy",
        Projection::At("/decisionStrategy"),
        Width::Fixed(18),
    )
    .toned(Tone::Muted),
    Column::new(
        "Description",
        Projection::At("/description"),
        Width::Flex(4),
    )
    .toned(Tone::Muted),
];
