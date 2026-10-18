use super::prelude::*;

pub(crate) const CLIENT_ACTIONS: &[Action] = &[
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
        "show-rotated-secret",
        "Show rotated client secret",
        Verb::Fetch("client-secret/rotated"),
        Danger::Safe,
    )
    .needs(Requirement::Feature(features::CLIENT_SECRET_ROTATION))
    .keywords(&["secret rotation", "previous secret", "grace period"]),
    Action::new(
        "invalidate-rotated-secret",
        "Invalidate rotated client secret",
        Verb::Remove("client-secret/rotated"),
        Danger::Destructive,
    )
    .needs(Requirement::Feature(features::CLIENT_SECRET_ROTATION))
    .keywords(&["secret rotation", "revoke old secret"]),
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
    Action::new(
        "scim-probe",
        "Probe SCIM as this client",
        Verb::Special("scim-probe"),
        Danger::Safe,
    )
    .needs(Requirement::Feature(features::SCIM_API))
    .keywords(&["scim", "provisioning", "test scim", "why does scim 401"]),
    Action::new(
        "scim-prepare",
        "Prepare client for SCIM",
        Verb::Special("scim-prepare"),
        Danger::Confirm,
    )
    .needs(Requirement::Feature(features::SCIM_API))
    .keywords(&["scim", "provisioning", "audience"]),
];
