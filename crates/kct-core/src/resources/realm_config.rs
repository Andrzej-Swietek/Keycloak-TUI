use super::prelude::*;

pub(crate) const REALM_KEYS: Resource = Resource {
    id: "realm-keys",
    singular: "Key",
    plural: "Keys",
    section: Section::Realm,
    collection: ep::KEYS,
    item: None,
    shape: Shape::ArrayAt("/keys"),
    fixed_params: &[],
    key: Key::custom("kid"),
    new_object: None,
    label: &["/kid"],
    columns: &[
        Column::new("Algorithm", Projection::At("/algorithm"), Width::Fixed(12))
            .toned(Tone::Primary),
        Column::new("Use", Projection::At("/use"), Width::Fixed(8)),
        Column::new("Status", Projection::At("/status"), Width::Fixed(10)).toned(Tone::Flag),
        Column::new("Type", Projection::At("/type"), Width::Fixed(8)),
        Column::new("Provider", Projection::At("/providerId"), Width::Flex(3)).toned(Tone::Muted),
        Column::new("Kid", Projection::At("/kid"), Width::Flex(4)).toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::RO,
    requirement: Requirement::Always,
    tabs: &[Tab::Raw],
    actions: &[],
    keywords: &[
        "jwks",
        "signing key",
        "rsa",
        "rotation",
        "certificate",
        "kid",
    ],
    summary: "Active, passive and disabled keys this realm signs with.",
};

pub(crate) const KEY_PROVIDERS: Resource = Resource {
    id: "key-providers",
    singular: "Key provider",
    plural: "Key providers",
    section: Section::Realm,
    collection: ep::COMPONENTS,
    item: Some(ep::COMPONENT),
    shape: Shape::Array,
    fixed_params: &[("type", "org.keycloak.keys.KeyProvider")],
    key: Key::ID,
    new_object: Some(new_object::key_provider),
    label: &["/name", "/id"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
        Column::new("Provider", Projection::At("/providerId"), Width::Fixed(16)),
        Column::new(
            "Priority",
            Projection::At("/config/priority/0"),
            Width::Fixed(9),
        ),
        Column::new(
            "Algorithm",
            Projection::At("/config/algorithm/0"),
            Width::Fixed(12),
        ),
        Column::new(
            "Enabled",
            Projection::At("/config/enabled/0"),
            Width::Fixed(8),
        )
        .toned(Tone::Flag),
        Column::new(
            "Active",
            Projection::At("/config/active/0"),
            Width::Fixed(8),
        ),
    ],
    search: Search::Client,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Details",
            fields: &[
                Field::new("Name", "/name", FieldKind::Text),
                Field::new("Provider", "/providerId", FieldKind::ReadOnly),
                Field::config("Priority", "/config/priority", FieldKind::Integer),
                Field::config("Enabled", "/config/enabled", FieldKind::Bool),
                Field::config("Active", "/config/active", FieldKind::Bool),
                Field::config("Algorithm", "/config/algorithm", FieldKind::Text),
                Field::config("Key size", "/config/keySize", FieldKind::Integer),
            ],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new("create", "Add key provider", Verb::Create, Danger::Confirm),
        Action::new("edit", "Edit key provider", Verb::Edit, Danger::Confirm),
        Action::new(
            "delete",
            "Delete key provider",
            Verb::Delete,
            Danger::Destructive,
        )
        .keywords(&["rotate keys"]),
    ],
    keywords: &["key rotation", "rsa generated", "hmac", "java keystore"],
    summary: "Providers that supply the realm's signing and encryption keys.",
};

pub(crate) const USER_PROFILE: Resource = Resource {
    id: "user-profile",
    singular: "User profile",
    plural: "User profile",
    section: Section::Realm,
    collection: ep::USER_PROFILE,
    item: Some(ep::USER_PROFILE),
    shape: Shape::Singleton,
    fixed_params: &[],
    key: Key::custom("__singleton"),
    new_object: None,
    label: &[],
    columns: &[],
    search: Search::None,
    ops: Ops::RW,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Policy",
            fields: &[Field::new(
                "Unmanaged attribute policy",
                "/unmanagedAttributePolicy",
                FieldKind::Choice(&["ENABLED", "ADMIN_VIEW", "ADMIN_EDIT", "DISABLED"]),
            )
            .help(
                "What happens to attributes the profile does not declare. Unset since \
                     Keycloak 24 means DISABLED: writes are accepted and silently discarded.",
            )],
        },
        // The two arrays inside this document are collections in their own right; a form field
        // holding either of them whole is a JSON blob nobody can edit.
        Tab::Nested {
            title: "Attributes",
            resource: "user-profile-attributes",
            slot: "",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Groups",
            resource: "user-profile-groups",
            slot: "",
            source: None,
            preset: &[],
        },
        Tab::Raw,
    ],
    actions: &[Action::new(
        "edit",
        "Edit user profile",
        Verb::Edit,
        Danger::Confirm,
    )],
    keywords: &[
        "declarative user profile",
        "attribute validation",
        "required attributes",
    ],
    summary: "Declarative definition of user attributes, validators and permissions.",
};
