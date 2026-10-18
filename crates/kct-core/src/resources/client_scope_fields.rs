//! What reaches a client's tokens, and which flows it authenticates with.

use super::prelude::*;

pub(crate) const CLIENT_SCOPE_FIELDS: &[Field] = &[
    Field::new("Full scope allowed", "/fullScopeAllowed", FieldKind::Bool).help(
        "On, every role the user holds reaches this client's tokens. Off, only the roles its \
         scope mappings name. Deprecated in 26.8: each token issued with it on logs a WARN, \
         and the `full-scope-disabled` client-policy executor enforces off.",
    ),
    Field::new(
        "Browser flow override",
        "/authenticationFlowBindingOverrides/browser",
        FieldKind::Text,
    )
    .help("Flow id, not alias. Empty uses the realm's binding."),
    Field::new(
        "Direct grant flow override",
        "/authenticationFlowBindingOverrides/direct_grant",
        FieldKind::Text,
    ),
    Field::new(
        "Node re-registration (s)",
        "/nodeReRegistrationTimeout",
        FieldKind::Seconds,
    ),
    Field::new(
        "Surrogate auth required",
        "/surrogateAuthRequired",
        FieldKind::Bool,
    ),
];
