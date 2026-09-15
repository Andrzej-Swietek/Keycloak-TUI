use super::prelude::*;

/// Public / confidential / bearer-only is not a field: it is implied by three booleans, and
/// getting it wrong is the single most common Keycloak misconfiguration. Compute it once.
pub(crate) fn client_kind(row: &Value) -> String {
    let flag = |name: &str| row.get(name).and_then(Value::as_bool).unwrap_or(false);
    let base = if flag("bearerOnly") {
        "bearer-only"
    } else if flag("publicClient") {
        "public"
    } else {
        "confidential"
    };
    if flag("serviceAccountsEnabled") {
        format!("{base} +svc")
    } else {
        base.to_owned()
    }
}

/// Which OAuth flows the client may use, abbreviated for a narrow column.
pub(crate) fn client_flows(row: &Value) -> String {
    let flag = |name: &str| row.get(name).and_then(Value::as_bool).unwrap_or(false);
    [
        ("std", flag("standardFlowEnabled")),
        ("implicit", flag("implicitFlowEnabled")),
        ("direct", flag("directAccessGrantsEnabled")),
        ("authz", flag("authorizationServicesEnabled")),
    ]
    .into_iter()
    .filter_map(|(label, on)| on.then_some(label))
    .collect::<Vec<_>>()
    .join(",")
}

pub(crate) const CLIENT_COLUMNS: &[Column] = &[
    Column::new("Client ID", Projection::At("/clientId"), Width::Flex(4)).toned(Tone::Primary),
    Column::new("Name", Projection::At("/name"), Width::Flex(3)),
    Column::new("Type", Projection::Compute(client_kind), Width::Fixed(18)),
    Column::new("Flows", Projection::Compute(client_flows), Width::Fixed(22)).toned(Tone::Muted),
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
    Column::new("Protocol", Projection::At("/protocol"), Width::Fixed(14)).toned(Tone::Muted),
];

pub(crate) const CLIENT_SETTINGS_FIELDS: &[Field] = &[
    Field::new("Internal ID", "/id", FieldKind::ReadOnly),
    Field::new("Client ID", "/clientId", FieldKind::Text),
    Field::new("Name", "/name", FieldKind::Text),
    Field::new("Description", "/description", FieldKind::Multiline),
    Field::new("Enabled", "/enabled", FieldKind::Bool),
    Field::new(
        "Protocol",
        "/protocol",
        FieldKind::Choice(&["openid-connect", "saml"]),
    ),
    Field::new(
        "Always display in console",
        "/alwaysDisplayInConsole",
        FieldKind::Bool,
    ),
    Field::new("Consent required", "/consentRequired", FieldKind::Bool),
];

pub(crate) const CLIENT_ACCESS_FIELDS: &[Field] = &[
    Field::new("Root URL", "/rootUrl", FieldKind::Text),
    Field::new("Base URL", "/baseUrl", FieldKind::Text),
    Field::new("Admin URL", "/adminUrl", FieldKind::Text)
        .help("Where Keycloak pushes back-channel logout and revocation."),
    Field::new("Redirect URIs", "/redirectUris", FieldKind::List)
        .help("Exact matches only. A trailing `*` is an open-redirect risk — avoid it."),
    Field::stringly(
        "Post-logout redirect URIs",
        "/attributes/post.logout.redirect.uris",
        FieldKind::Text,
    )
    .help("`+` reuses the redirect URIs; `-` disables post-logout redirects."),
    Field::new("Web origins", "/webOrigins", FieldKind::List)
        .help("CORS origins. `+` mirrors the redirect URIs; `*` allows any origin."),
];

pub(crate) const CLIENT_CAPABILITY_FIELDS: &[Field] = &[
    Field::new("Public client", "/publicClient", FieldKind::Bool)
        .help("Public clients hold no secret and MUST use PKCE."),
    Field::new("Bearer only", "/bearerOnly", FieldKind::Bool),
    Field::new(
        "Client authenticator",
        "/clientAuthenticatorType",
        FieldKind::Choice(&[
            "client-secret",
            "client-jwt",
            "client-secret-jwt",
            "client-x509",
        ]),
    ),
    Field::new("Standard flow", "/standardFlowEnabled", FieldKind::Bool)
        .help("Authorization Code flow."),
    Field::new("Implicit flow", "/implicitFlowEnabled", FieldKind::Bool)
        .help("Deprecated by OAuth 2.1 — prefer code + PKCE."),
    Field::new(
        "Direct access grants",
        "/directAccessGrantsEnabled",
        FieldKind::Bool,
    )
    .help("Resource-owner password grant. Needed by scripts and by this tool's password login."),
    Field::new(
        "Service accounts",
        "/serviceAccountsEnabled",
        FieldKind::Bool,
    )
    .help("Enables client_credentials and gives the client its own user."),
    Field::new(
        "Authorization services",
        "/authorizationServicesEnabled",
        FieldKind::Bool,
    )
    .needs(Requirement::Feature(features::AUTHORIZATION)),
    Field::stringly(
        "PKCE challenge method",
        "/attributes/pkce.code.challenge.method",
        FieldKind::Choice(&["", "S256", "plain"]),
    )
    .help("Set S256 on every public client."),
    Field::new(
        "Front-channel logout",
        "/frontchannelLogout",
        FieldKind::Bool,
    ),
];

pub(crate) const CLIENT_TOKEN_FIELDS: &[Field] = &[
    Field::stringly(
        "Access token lifespan",
        "/attributes/access.token.lifespan",
        FieldKind::Seconds,
    ),
    Field::stringly(
        "Client session idle",
        "/attributes/client.session.idle.timeout",
        FieldKind::Seconds,
    ),
    Field::stringly(
        "Client session max",
        "/attributes/client.session.max.lifespan",
        FieldKind::Seconds,
    ),
    Field::stringly(
        "Client offline idle",
        "/attributes/client.offline.session.idle.timeout",
        FieldKind::Seconds,
    ),
    Field::stringly(
        "Use refresh tokens",
        "/attributes/use.refresh.tokens",
        FieldKind::Bool,
    ),
    Field::stringly(
        "Lightweight access token",
        "/attributes/client.use.lightweight.access.token.enabled",
        FieldKind::Bool,
    )
    .help("Strips non-essential claims from the access token."),
    Field::stringly(
        "Signature algorithm",
        "/attributes/access.token.signed.response.alg",
        FieldKind::Text,
    ),
];
