use super::prelude::*;

pub(crate) const USER_CONSENTS: Resource = Resource {
    id: "user-consents",
    singular: "Consent",
    plural: "Consents",
    section: Section::Manage,
    collection: ep::USER_CONSENTS,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::custom("clientId"),
    new_object: None,
    label: &["/clientId"],
    columns: &[
        Column::new("Client", Projection::At("/clientId"), Width::Flex(3)).toned(Tone::Primary),
        Column::new(
            "Granted scopes",
            Projection::Compute(consent_scopes),
            Width::Flex(6),
        ),
        Column::new(
            "Created",
            Projection::EpochMillis("/createdDate"),
            Width::Fixed(19),
        ),
    ],
    search: Search::Client,
    ops: Ops::RO.with(Ops::DELETE),
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[Action::new(
        "delete",
        "Revoke consent",
        Verb::Delete,
        Danger::Confirm,
    )],
    keywords: &["granted", "scope consent", "revoke"],
    summary: "Scopes this user has consented to, per client.",
};

pub(crate) const USER_IDENTITY_LINKS: Resource = Resource {
    id: "user-identity-links",
    singular: "Identity link",
    plural: "Identity links",
    section: Section::Manage,
    collection: ep::USER_FEDERATED_IDENTITY,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::custom("identityProvider"),
    new_object: None,
    label: &["/identityProvider"],
    columns: &[
        Column::new(
            "Provider",
            Projection::At("/identityProvider"),
            Width::Flex(2),
        )
        .toned(Tone::Primary),
        Column::new("External user", Projection::At("/userName"), Width::Flex(3)),
        Column::new("External ID", Projection::At("/userId"), Width::Flex(3)).toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::RO.with(Ops::DELETE),
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[Action::new(
        "delete",
        "Unlink provider",
        Verb::Delete,
        Danger::Confirm,
    )],
    keywords: &["broker link", "federated identity", "social"],
    summary: "External accounts linked to this user.",
};

pub(crate) fn consent_scopes(row: &Value) -> String {
    row.get("grantedClientScopes")
        .and_then(Value::as_array)
        .map(|scopes| {
            scopes
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}
