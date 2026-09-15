//! Passkey and security-key policy.
//!
//! Keycloak keeps two independent policies with the same field names: `webAuthnPolicy*` for
//! two-factor, `webAuthnPolicyPasswordless*` for passwordless sign-in. Changing one leaves
//! the other alone.

use super::prelude::*;

/// Values Keycloak accepts for the attestation, attachment and verification settings.
const ATTESTATION: &[&str] = &["none", "indirect", "direct"];
const ATTACHMENT: &[&str] = &["platform", "cross-platform", "not specified"];
const VERIFICATION: &[&str] = &["required", "preferred", "discouraged"];
const RESIDENT_KEY: &[&str] = &["Yes", "No", "not specified"];

pub(crate) const WEBAUTHN_FIELDS: &[Field] = &[
    Field::new(
        "Relying party name",
        "/webAuthnPolicyRpEntityName",
        FieldKind::Text,
    )
    .help("Shown by the authenticator when it asks the user to confirm."),
    Field::new("Relying party id", "/webAuthnPolicyRpId", FieldKind::Text)
        .help("The site's domain. Empty means the realm's own hostname."),
    Field::new(
        "Signature algorithms",
        "/webAuthnPolicySignatureAlgorithms",
        FieldKind::List,
    ),
    Field::new(
        "Attestation conveyance",
        "/webAuthnPolicyAttestationConveyancePreference",
        FieldKind::Choice(ATTESTATION),
    ),
    Field::new(
        "Authenticator attachment",
        "/webAuthnPolicyAuthenticatorAttachment",
        FieldKind::Choice(ATTACHMENT),
    ),
    Field::new(
        "Require resident key",
        "/webAuthnPolicyRequireResidentKey",
        FieldKind::Choice(RESIDENT_KEY),
    ),
    Field::new(
        "Resident key",
        "/webAuthnPolicyResidentKey",
        FieldKind::Choice(RESIDENT_KEY),
    ),
    Field::new(
        "User verification",
        "/webAuthnPolicyUserVerificationRequirement",
        FieldKind::Choice(VERIFICATION),
    ),
    Field::new(
        "Registration timeout (s)",
        "/webAuthnPolicyCreateTimeout",
        FieldKind::Seconds,
    ),
    Field::new(
        "One registration per authenticator",
        "/webAuthnPolicyAvoidSameAuthenticatorRegister",
        FieldKind::Bool,
    ),
    Field::new(
        "Acceptable AAGUIDs",
        "/webAuthnPolicyAcceptableAaguids",
        FieldKind::List,
    )
    .help("Authenticator models allowed to register. Empty allows any."),
    Field::new(
        "Extra origins",
        "/webAuthnPolicyExtraOrigins",
        FieldKind::List,
    ),
];

pub(crate) const WEBAUTHN_PASSWORDLESS_FIELDS: &[Field] = &[
    Field::new(
        "Relying party name",
        "/webAuthnPolicyPasswordlessRpEntityName",
        FieldKind::Text,
    ),
    Field::new(
        "Relying party id",
        "/webAuthnPolicyPasswordlessRpId",
        FieldKind::Text,
    ),
    Field::new(
        "Signature algorithms",
        "/webAuthnPolicyPasswordlessSignatureAlgorithms",
        FieldKind::List,
    ),
    Field::new(
        "Attestation conveyance",
        "/webAuthnPolicyPasswordlessAttestationConveyancePreference",
        FieldKind::Choice(ATTESTATION),
    ),
    Field::new(
        "Authenticator attachment",
        "/webAuthnPolicyPasswordlessAuthenticatorAttachment",
        FieldKind::Choice(ATTACHMENT),
    ),
    Field::new(
        "Require resident key",
        "/webAuthnPolicyPasswordlessRequireResidentKey",
        FieldKind::Choice(RESIDENT_KEY),
    )
    .help("Passwordless sign-in needs a resident key; `No` makes the policy two-factor only."),
    Field::new(
        "Resident key",
        "/webAuthnPolicyPasswordlessResidentKey",
        FieldKind::Choice(RESIDENT_KEY),
    ),
    Field::new(
        "User verification",
        "/webAuthnPolicyPasswordlessUserVerificationRequirement",
        FieldKind::Choice(VERIFICATION),
    ),
    Field::new(
        "Registration timeout (s)",
        "/webAuthnPolicyPasswordlessCreateTimeout",
        FieldKind::Seconds,
    ),
    Field::new(
        "One registration per authenticator",
        "/webAuthnPolicyPasswordlessAvoidSameAuthenticatorRegister",
        FieldKind::Bool,
    ),
    Field::new(
        "Acceptable AAGUIDs",
        "/webAuthnPolicyPasswordlessAcceptableAaguids",
        FieldKind::List,
    ),
    Field::new(
        "Extra origins",
        "/webAuthnPolicyPasswordlessExtraOrigins",
        FieldKind::List,
    ),
];
