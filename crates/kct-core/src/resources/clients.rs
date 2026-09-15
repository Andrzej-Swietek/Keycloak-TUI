use super::client_fields::{
    CLIENT_ACCESS_FIELDS, CLIENT_CAPABILITY_FIELDS, CLIENT_COLUMNS, CLIENT_SETTINGS_FIELDS,
    CLIENT_TOKEN_FIELDS,
};
use super::prelude::*;

pub(crate) const CLIENTS: Resource = Resource {
    id: "clients",
    singular: "Client",
    plural: "Clients",
    section: Section::Configure,
    collection: ep::CLIENTS,
    item: Some(ep::CLIENT),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::client),
    label: &["/clientId", "/name", "/id"],
    columns: CLIENT_COLUMNS,
    search: Search::Server,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Settings",
            fields: CLIENT_SETTINGS_FIELDS,
        },
        Tab::Form {
            title: "Access",
            fields: CLIENT_ACCESS_FIELDS,
        },
        Tab::Form {
            title: "Capability",
            fields: CLIENT_CAPABILITY_FIELDS,
        },
        Tab::Form {
            title: "Tokens",
            fields: CLIENT_TOKEN_FIELDS,
        },
        Tab::Nested {
            title: "Roles",
            resource: "client-roles",
            slot: "client",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Mappers",
            resource: "client-mappers",
            slot: "client",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Default scopes",
            resource: "client-default-scopes",
            slot: "client",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Optional scopes",
            resource: "client-optional-scopes",
            slot: "client",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Effective mappers",
            resource: "client-effective-mappers",
            slot: "client",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Sessions",
            resource: "client-sessions",
            slot: "client",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Authz resources",
            resource: "client-authz-resources",
            slot: "client",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Authz scopes",
            resource: "client-authz-scopes",
            slot: "client",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Authz policies",
            resource: "client-authz-policies",
            slot: "client",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Authz permissions",
            resource: "client-authz-permissions",
            slot: "client",
            source: None,
            preset: &[],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new("create", "Create client", Verb::Create, Danger::Confirm),
        Action::new(
            "add-client-scope",
            "Add default client scope",
            Verb::Special("add-client-scope"),
            Danger::Confirm,
        )
        .keywords(&["scope", "claims", "assign scope"]),
        Action::new("edit", "Edit client", Verb::Edit, Danger::Confirm),
        Action::new("delete", "Delete client", Verb::Delete, Danger::Destructive),
        Action::new(
            "show-secret",
            "Show client secret",
            Verb::Fetch("client-secret"),
            Danger::Safe,
        )
        .keywords(&["credential", "password"]),
        Action::new(
            "regenerate-secret",
            "Regenerate client secret",
            Verb::Special("regenerate-secret"),
            Danger::Destructive,
        )
        .keywords(&["rotate", "new secret"]),
        Action::new(
            "adapter-config",
            "Download adapter config",
            Verb::Special("installation-provider"),
            Danger::Safe,
        )
        .keywords(&["keycloak.json", "installation", "jboss", "saml descriptor"]),
        Action::new(
            "service-account",
            "Open service account user",
            Verb::Special("service-account-user"),
            Danger::Safe,
        ),
        Action::new(
            "get-token",
            "Get a token as this client",
            Verb::Special("token-for-client"),
            Danger::Safe,
        )
        .keywords(&[
            "client credentials",
            "test integration",
            "what claims does it get",
            "jwt",
            "why is my role missing",
        ]),
    ],
    keywords: &["application", "app", "oidc", "saml", "relying party", "sp"],
    summary: "Applications and services that obtain tokens from this realm.",
};
