use super::prelude::*;

pub(crate) const IDENTITY_PROVIDERS: Resource = Resource {
    id: "identity-providers",
    singular: "Identity provider",
    plural: "Identity providers",
    section: Section::Configure,
    collection: ep::IDENTITY_PROVIDERS,
    item: Some(ep::IDENTITY_PROVIDER),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ALIAS,
    new_object: Some(new_object::identity_provider),
    label: &["/displayName", "/alias"],
    columns: &[
        Column::new("Alias", Projection::At("/alias"), Width::Flex(3)).toned(Tone::Primary),
        Column::new("Provider", Projection::At("/providerId"), Width::Fixed(16)),
        Column::new("Display name", Projection::At("/displayName"), Width::Flex(3)),
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
            "Trust email",
            Projection::Bool {
                pointer: "/trustEmail",
                yes: "yes",
                no: "no",
            },
            Width::Fixed(11),
        )
        .toned(Tone::Warn),
        Column::new(
            "First login flow",
            Projection::At("/firstBrokerLoginFlowAlias"),
            Width::Flex(3),
        )
        .toned(Tone::Muted),
    ],
    search: Search::Client,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Details",
            fields: &[
                Field::new("Alias", "/alias", FieldKind::Text),
                Field::new("Display name", "/displayName", FieldKind::Text),
                Field::new("Provider", "/providerId", FieldKind::ReadOnly),
                Field::new("Enabled", "/enabled", FieldKind::Bool),
                Field::new("Trust email", "/trustEmail", FieldKind::Bool)
                    .help("Skips e-mail verification for users from this IdP. Only enable for an IdP you control."),
                Field::new("Store tokens", "/storeToken", FieldKind::Bool),
                Field::new("Account linking only", "/linkOnly", FieldKind::Bool),
                Field::new("Hide on login page", "/hideOnLogin", FieldKind::Bool),
                Field::new("Organizations", "/organizationLinks", FieldKind::ReadOnly)
                    .help("Managed from each organization's Identity providers tab.")
                    .needs(KC_26_8),
                Field::new("Organization", "/organizationId", FieldKind::ReadOnly)
                    .help("Before 26.8 an IdP belonged to at most one organization."),
                Field::new("First broker login flow", "/firstBrokerLoginFlowAlias", FieldKind::Text),
                Field::new("Post broker login flow", "/postBrokerLoginFlowAlias", FieldKind::Text),
                Field::stringly("Allow admin role mapping", "/config/allowAdminRoleMapping", FieldKind::Bool)
                    .help("From 26.8, mappers on this IdP cannot grant realm-management or other admin roles unless this is on. Needs `manage-realm`.")
                    .needs(KC_26_8),
            ],
        },
        Tab::Form {
            title: "Endpoints",
            fields: &[
                Field::new("Authorization URL", "/config/authorizationUrl", FieldKind::Text),
                Field::new("Token URL", "/config/tokenUrl", FieldKind::Text),
                Field::new("User info URL", "/config/userInfoUrl", FieldKind::Text),
                Field::new("Issuer", "/config/issuer", FieldKind::Text),
                Field::new("JWKS URL", "/config/jwksUrl", FieldKind::Text),
                Field::new("Logout URL", "/config/logoutUrl", FieldKind::Text),
                Field::new("Client ID", "/config/clientId", FieldKind::Text),
                Field::new("Client secret", "/config/clientSecret", FieldKind::Secret)
                    .help("Shown masked. From 26.8, changing the token URL, client ID or auth method needs the secret typed again."),
                Field::new(
                    "Client auth method",
                    "/config/clientAuthMethod",
                    FieldKind::Choice(&[
                        "client_secret_post",
                        "client_secret_basic",
                        "client_secret_jwt",
                        "private_key_jwt",
                    ]),
                ),
                Field::new("Default scopes", "/config/defaultScope", FieldKind::Text),
                Field::stringly("Validate signatures", "/config/validateSignature", FieldKind::Bool),
                Field::stringly("PKCE enabled", "/config/pkceEnabled", FieldKind::Bool),
            ],
        },
        Tab::Nested {
            title: "Mappers",
            resource: "idp-mappers",
            slot: "alias",
            source: None,
            preset: &[],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new("create", "Create identity provider", Verb::Create, Danger::Confirm),
        Action::new("edit", "Edit identity provider", Verb::Edit, Danger::Confirm),
        Action::new("delete", "Delete identity provider", Verb::Delete, Danger::Destructive),
    ],
    keywords: &["idp", "broker", "sso", "federation", "social", "oidc", "saml", "google", "azure"],
    summary: "External identity providers this realm brokers logins from.",
};

pub(crate) const IDP_MAPPERS: Resource = Resource {
    id: "idp-mappers",
    singular: "IdP mapper",
    plural: "IdP mappers",
    section: Section::Configure,
    collection: ep::IDENTITY_PROVIDER_MAPPERS,
    item: Some(ep::IDENTITY_PROVIDER_MAPPER),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::idp_mapper),
    label: &["/name"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
        Column::new(
            "Type",
            Projection::At("/identityProviderMapper"),
            Width::Flex(4),
        ),
        Column::new(
            "Sync mode",
            Projection::At("/config/syncMode"),
            Width::Fixed(12),
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
                Field::new("Mapper type", "/identityProviderMapper", FieldKind::Text),
                Field::new(
                    "Sync mode",
                    "/config/syncMode",
                    FieldKind::Choice(&["INHERIT", "IMPORT", "LEGACY", "FORCE"]),
                ),
                Field::new("Config", "/config", FieldKind::StringMap)
                    .help("Endpoint URLs, client credentials and per-provider switches."),
            ],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new("create", "Add mapper", Verb::Create, Danger::Confirm),
        Action::new("delete", "Delete mapper", Verb::Delete, Danger::Confirm),
    ],
    keywords: &["claim", "attribute import", "role import"],
    summary: "How claims from the external IdP land on the brokered user.",
};
