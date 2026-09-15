//! Policies applied to clients created through the registration service.
//!
//! Components under one SPI, split by `subType`: `anonymous` governs registration with an
//! initial access token, `authenticated` governs a client editing itself with its registration
//! access token. Without these, self-service registration is unconstrained.

use super::prelude::*;

const POLICY_TYPE: &str =
    "org.keycloak.services.clientregistration.policy.ClientRegistrationPolicy";

pub(crate) const CLIENT_REGISTRATION_POLICIES: Resource = Resource {
    id: "client-registration-policies",
    singular: "Registration policy",
    plural: "Client registration policies",
    section: Section::Configure,
    collection: ep::COMPONENTS,
    item: Some(ep::COMPONENT),
    shape: Shape::Array,
    fixed_params: &[("type", POLICY_TYPE)],
    key: Key::ID,
    new_object: None,
    label: &["/name"],
    columns: &[
        Column::new("Name", Projection::At("/name"), Width::Flex(3)).toned(Tone::Primary),
        Column::new("Applies to", Projection::At("/subType"), Width::Fixed(14)).toned(Tone::Flag),
        Column::new("Provider", Projection::At("/providerId"), Width::Flex(3)).toned(Tone::Muted),
        Column::new("Settings", Projection::Count("/config"), Width::Fixed(9)),
    ],
    search: Search::Client,
    ops: Ops::RW,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Policy",
            fields: &[
                Field::new("Name", "/name", FieldKind::ReadOnly),
                Field::new("Provider", "/providerId", FieldKind::ReadOnly),
                Field::new("Applies to", "/subType", FieldKind::ReadOnly)
                    .help("`anonymous` covers registration with an initial access token."),
                Field::new("Config", "/config", FieldKind::Attributes)
                    .help("Keys the provider advertises; every value is a list."),
            ],
        },
        Tab::Raw,
    ],
    actions: &[
        Action::new(
            "edit",
            "Edit registration policy",
            Verb::Edit,
            Danger::Confirm,
        ),
        Action::new(
            "delete",
            "Delete registration policy",
            Verb::Delete,
            Danger::Destructive,
        ),
    ],
    keywords: &[
        "dynamic client registration",
        "initial access token",
        "trusted hosts",
        "max clients",
        "self service",
    ],
    summary: "What a client may ask for when it registers itself.",
};
