use super::ldap_fields::{
    LDAP_CONNECTION_FIELDS, LDAP_KERBEROS_FIELDS, LDAP_SEARCH_FIELDS, LDAP_SYNC_FIELDS,
};
use super::prelude::*;

pub(crate) const USER_FEDERATION_COLUMNS: &[Column] = &[
    Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
    Column::new("Provider", Projection::At("/providerId"), Width::Fixed(12)),
    Column::new(
        "Enabled",
        Projection::At("/config/enabled/0"),
        Width::Fixed(8),
    )
    .toned(Tone::Flag),
    Column::new(
        "Edit mode",
        Projection::At("/config/editMode/0"),
        Width::Fixed(11),
    ),
    Column::new(
        "Connection",
        Projection::At("/config/connectionUrl/0"),
        Width::Flex(4),
    ),
    Column::new(
        "Import",
        Projection::At("/config/importEnabled/0"),
        Width::Fixed(8),
    )
    .toned(Tone::Muted),
];

pub(crate) const USER_FEDERATION: Resource = Resource {
    id: "user-federation",
    singular: "User federation provider",
    plural: "User federation",
    section: Section::Configure,
    collection: ep::COMPONENTS,
    item: Some(ep::COMPONENT),
    shape: Shape::Array,
    // `/components` serves every SPI; without the type filter this lists all of them.
    fixed_params: &[("type", "org.keycloak.storage.UserStorageProvider")],
    key: Key::ID,
    new_object: Some(new_object::ldap_provider),
    label: &["/name", "/id"],
    columns: USER_FEDERATION_COLUMNS,
    search: Search::Client,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Connection",
            fields: LDAP_CONNECTION_FIELDS,
        },
        Tab::Form {
            title: "Searching",
            fields: LDAP_SEARCH_FIELDS,
        },
        Tab::Form {
            title: "Sync",
            fields: LDAP_SYNC_FIELDS,
        },
        Tab::Form {
            title: "Kerberos",
            fields: LDAP_KERBEROS_FIELDS,
        },
        Tab::Nested {
            title: "Mappers",
            resource: "federation-mappers",
            slot: "component",
            source: None,
            preset: &[],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new("create", "Add provider", Verb::Create, Danger::Confirm),
        Action::new("edit", "Edit provider", Verb::Edit, Danger::Confirm),
        Action::new(
            "delete",
            "Delete provider",
            Verb::Delete,
            Danger::Destructive,
        )
        .keywords(&["remove ldap"]),
        Action::new(
            "test-connection",
            "Test connection",
            Verb::Special("ldap-test-connection"),
            Danger::Safe,
        )
        .keywords(&["ping ldap", "check", "tls"]),
        Action::new(
            "test-authentication",
            "Test authentication (bind)",
            Verb::Special("ldap-test-authentication"),
            Danger::Safe,
        )
        .keywords(&["bind dn", "credentials"]),
        Action::new(
            "sync-full",
            "Sync all users",
            Verb::Special("ldap-sync-full"),
            Danger::Confirm,
        )
        .keywords(&["import", "full sync"]),
        Action::new(
            "sync-changed",
            "Sync changed users",
            Verb::Special("ldap-sync-changed"),
            Danger::Confirm,
        ),
        Action::new(
            "remove-imported",
            "Remove imported users",
            Verb::Special("ldap-remove-imported"),
            Danger::Destructive,
        )
        .keywords(&["purge", "cleanup"]),
        Action::new(
            "unlink-users",
            "Unlink users",
            Verb::Special("ldap-unlink-users"),
            Danger::Destructive,
        ),
    ],
    keywords: &[
        "ldap",
        "active directory",
        "ad",
        "kerberos",
        "storage",
        "directory",
        "sync",
    ],
    summary: "LDAP, Active Directory and Kerberos providers backing this realm's users.",
};

pub(crate) const FEDERATION_MAPPERS: Resource = Resource {
    id: "federation-mappers",
    singular: "Federation mapper",
    plural: "Federation mappers",
    section: Section::Configure,
    collection: ep::COMPONENTS,
    item: Some(ep::COMPONENT),
    shape: Shape::Array,
    fixed_params: &[
        ("parent", "{component}"),
        (
            "type",
            "org.keycloak.storage.ldap.mappers.LDAPStorageMapper",
        ),
    ],
    key: Key::ID,
    new_object: Some(new_object::ldap_mapper),
    label: &["/name", "/id"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
        Column::new("Type", Projection::At("/providerId"), Width::Flex(3)),
        Column::new(
            "LDAP attribute",
            Projection::At("/config/ldap.attribute/0"),
            Width::Flex(2),
        ),
        Column::new(
            "User attribute",
            Projection::At("/config/user.model.attribute/0"),
            Width::Flex(2),
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
                Field::new("Mapper type", "/providerId", FieldKind::ReadOnly),
                Field::new("Config", "/config", FieldKind::Attributes)
                    .help("Component config: every value is a list, even single ones."),
            ],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new("create", "Add mapper", Verb::Create, Danger::Confirm),
        Action::new("delete", "Delete mapper", Verb::Delete, Danger::Confirm),
        Action::new(
            "sync",
            "Run mapper sync",
            Verb::Special("ldap-mapper-sync"),
            Danger::Confirm,
        )
        .keywords(&["fed to keycloak", "keycloak to fed"]),
    ],
    keywords: &["ldap mapper", "group mapper", "role mapper", "attribute"],
    summary: "How LDAP entries map onto Keycloak users, groups and roles.",
};
