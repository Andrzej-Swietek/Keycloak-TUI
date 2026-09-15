use super::prelude::*;
use super::realm_fields::{REALM_GENERAL_FIELDS, REALM_LOGIN_FIELDS, REALM_SECURITY_FIELDS};
use super::realm_runtime_fields::{
    REALM_EMAIL_FIELDS, REALM_SESSION_FIELDS, REALM_THEME_FIELDS, REALM_TOKEN_FIELDS,
};

pub(crate) const REALM_SETTINGS: Resource = Resource {
    id: "realm-settings",
    singular: "Realm settings",
    plural: "Realm settings",
    section: Section::Realm,
    collection: ep::REALM,
    item: Some(ep::REALM),
    shape: Shape::Singleton,
    fixed_params: &[],
    key: Key::custom("realm"),
    new_object: None,
    label: &["/realm"],
    columns: &[Column::new(
        "Realm",
        Projection::At("/realm"),
        Width::Flex(1),
    )],
    search: Search::None,
    ops: Ops::RW,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "General",
            fields: REALM_GENERAL_FIELDS,
        },
        Tab::Form {
            title: "Login",
            fields: REALM_LOGIN_FIELDS,
        },
        Tab::Form {
            title: "Sessions",
            fields: REALM_SESSION_FIELDS,
        },
        Tab::Form {
            title: "Tokens",
            fields: REALM_TOKEN_FIELDS,
        },
        Tab::Form {
            title: "Security",
            fields: REALM_SECURITY_FIELDS,
        },
        Tab::Form {
            title: "Flows & OTP",
            fields: super::realm_flow_fields::REALM_FLOW_FIELDS,
        },
        Tab::Form {
            title: "Passkeys",
            fields: super::realm_webauthn_fields::WEBAUTHN_PASSWORDLESS_FIELDS,
        },
        Tab::Form {
            title: "Security keys",
            fields: super::realm_webauthn_fields::WEBAUTHN_FIELDS,
        },
        Tab::Form {
            title: "Email",
            fields: REALM_EMAIL_FIELDS,
        },
        Tab::Form {
            title: "Themes",
            fields: REALM_THEME_FIELDS,
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new("edit", "Edit realm settings", Verb::Edit, Danger::Confirm),
        Action::new("export", "Partial export realm", Verb::Export, Danger::Safe)
            .keywords(&["backup", "dump", "json"]),
        Action::new(
            "import",
            "Partial import into realm",
            Verb::Import,
            Danger::Destructive,
        )
        .keywords(&["restore", "load", "json"]),
        Action::new(
            "logout-all",
            "Log out every session",
            Verb::Post("logout-all"),
            Danger::Destructive,
        ),
        Action::new(
            "push-revocation",
            "Push revocation policy",
            Verb::Post("push-revocation"),
            Danger::Confirm,
        ),
        Action::new(
            "clear-lockouts",
            "Clear all brute-force lockouts",
            Verb::Special("clear-brute-force"),
            Danger::Confirm,
        ),
    ],
    keywords: &[
        "realm config",
        "sso",
        "token lifespan",
        "password policy",
        "smtp",
        "theme",
        "brute force",
    ],
    summary: "Everything that configures the realm as a whole.",
};

pub(crate) const REALMS: Resource = Resource {
    id: "realms",
    singular: "Realm",
    plural: "Realms",
    section: Section::Server,
    collection: ep::REALMS,
    item: Some(ep::REALM),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::custom("realm"),
    new_object: Some(new_object::realm),
    label: &["/realm", "/displayName", "/id"],
    columns: &[
        Column::new("Realm", Projection::At("/realm"), Width::Flex(3)).toned(Tone::Primary),
        Column::new(
            "Display name",
            Projection::At("/displayName"),
            Width::Flex(3),
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
            "SSL required",
            Projection::At("/sslRequired"),
            Width::Fixed(13),
        )
        .toned(Tone::Warn),
        Column::new(
            "Brute force",
            Projection::Bool {
                pointer: "/bruteForceProtected",
                yes: "on",
                no: "off",
            },
            Width::Fixed(12),
        ),
    ],
    search: Search::Client,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "General",
            fields: REALM_GENERAL_FIELDS,
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new(
            "switch",
            "Switch to realm",
            Verb::Special("switch-realm"),
            Danger::Safe,
        )
        .keywords(&["use", "select", "change realm"]),
        Action::new("create", "Create realm", Verb::Create, Danger::Confirm),
        Action::new("delete", "Delete realm", Verb::Delete, Danger::Destructive)
            .keywords(&["drop", "destroy"]),
        Action::new("export", "Partial export", Verb::Export, Danger::Safe),
    ],
    keywords: &["tenant", "realm list", "switch"],
    summary: "Realms on this server.",
};
