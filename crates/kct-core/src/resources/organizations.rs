use super::prelude::*;

pub(crate) const ORGANIZATION_COLUMNS: &[Column] = &[
    Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
    Column::new("Alias", Projection::At("/alias"), Width::Flex(2)).toned(Tone::Muted),
    Column::new(
        "Domains",
        Projection::Compute(organization_domains),
        Width::Flex(4),
    ),
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
        "Description",
        Projection::At("/description"),
        Width::Flex(4),
    )
    .toned(Tone::Muted),
];

pub(crate) fn organization_domains(row: &Value) -> String {
    row.get("domains")
        .and_then(Value::as_array)
        .map(|domains| {
            domains
                .iter()
                .map(|d| {
                    let name = d.get("name").and_then(Value::as_str).unwrap_or_default();
                    match d.get("verified").and_then(Value::as_bool) {
                        Some(true) => format!("{name} ✓"),
                        _ => name.to_owned(),
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}

pub(crate) const ORGANIZATIONS: Resource = Resource {
    id: "organizations",
    singular: "Organization",
    plural: "Organizations",
    section: Section::Manage,
    collection: ep::ORGANIZATIONS,
    item: Some(ep::ORGANIZATION),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::organization),
    label: &["/name", "/alias", "/id"],
    columns: ORGANIZATION_COLUMNS,
    search: Search::Server,
    ops: Ops::CRUD,
    // GA in 26.0, but the realm must also opt in via `organizationsEnabled`; the server has
    // the final word, so an unknown answer still lets the operator look.
    requirement: Requirement::All(&[KC_26, Requirement::Feature(features::ORGANIZATION)]),
    tabs: &[
        Tab::Form {
            title: "Details",
            fields: &[
                Field::new("ID", "/id", FieldKind::ReadOnly),
                Field::new("Name", "/name", FieldKind::Text),
                Field::new("Alias", "/alias", FieldKind::Text)
                    .help("Immutable once set; used in tokens and the organization claim."),
                Field::new("Enabled", "/enabled", FieldKind::Bool),
                Field::new("Description", "/description", FieldKind::Multiline),
                Field::new("Redirect URL", "/redirectUrl", FieldKind::Text),
                Field::new("Domains", "/domains", FieldKind::Json)
                    .help("[{\"name\": \"example.com\", \"verified\": true}]"),
                Field::new("Attributes", "/attributes", FieldKind::Attributes),
            ],
        },
        Tab::Nested {
            title: "Members",
            resource: "org-members",
            slot: "org",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Identity providers",
            resource: "org-identity-providers",
            slot: "org",
            source: None,
            preset: &[],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new(
            "create",
            "Create organization",
            Verb::Create,
            Danger::Confirm,
        ),
        Action::new("edit", "Edit organization", Verb::Edit, Danger::Confirm),
        Action::new(
            "delete",
            "Delete organization",
            Verb::Delete,
            Danger::Destructive,
        ),
        Action::new(
            "invite",
            "Invite member",
            Verb::Special("invite-member"),
            Danger::Confirm,
        ),
    ],
    keywords: &["tenant", "multitenancy", "org", "customer"],
    summary: "In-realm tenants with their own domains, members and identity providers.",
};

pub(crate) const ORG_IDENTITY_PROVIDERS: Resource = Resource {
    id: "org-identity-providers",
    singular: "Organization IdP",
    plural: "Organization identity providers",
    section: Section::Manage,
    collection: ep::ORGANIZATION_IDPS,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ALIAS,
    new_object: None,
    label: &["/alias", "/displayName"],
    columns: &[
        Column::new("Alias", Projection::At("/alias"), Width::Flex(2)).toned(Tone::Primary),
        Column::new("Provider", Projection::At("/providerId"), Width::Fixed(14)),
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
            "Domain",
            Projection::At("/config/kc.org.domain"),
            Width::Flex(2),
        ),
    ],
    search: Search::Client,
    ops: Ops::RO.with(Ops::CREATE).with(Ops::DELETE),
    requirement: KC_26,
    tabs: &[Tab::Raw],
    actions: &[Action::new(
        "delete",
        "Unlink identity provider",
        Verb::Delete,
        Danger::Confirm,
    )],
    keywords: &["broker", "sso", "tenant idp"],
    summary: "Identity providers bound to this organization.",
};
