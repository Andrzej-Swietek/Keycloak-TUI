//! Flow bindings and the one-time-password policy.

use super::prelude::*;

pub(crate) const REALM_FLOW_FIELDS: &[Field] = &[
    Field::new("Browser flow", "/browserFlow", FieldKind::Text),
    Field::new("Registration flow", "/registrationFlow", FieldKind::Text),
    Field::new("Direct grant flow", "/directGrantFlow", FieldKind::Text),
    Field::new(
        "Reset credentials flow",
        "/resetCredentialsFlow",
        FieldKind::Text,
    ),
    Field::new(
        "Client authentication flow",
        "/clientAuthenticationFlow",
        FieldKind::Text,
    ),
    Field::new(
        "Docker authentication flow",
        "/dockerAuthenticationFlow",
        FieldKind::Text,
    ),
    Field::new(
        "First broker login flow",
        "/firstBrokerLoginFlow",
        FieldKind::Text,
    )
    .needs(KC_26),
    Field::new(
        "OTP policy type",
        "/otpPolicyType",
        FieldKind::Choice(&["totp", "hotp"]),
    ),
    Field::new(
        "OTP algorithm",
        "/otpPolicyAlgorithm",
        FieldKind::Choice(&["HmacSHA1", "HmacSHA256", "HmacSHA512"]),
    ),
    Field::new(
        "OTP digits",
        "/otpPolicyDigits",
        FieldKind::Choice(&["6", "8"]),
    ),
    Field::new("OTP period", "/otpPolicyPeriod", FieldKind::Seconds),
    Field::new(
        "OTP code reusable",
        "/otpPolicyCodeReusable",
        FieldKind::Bool,
    )
    .help("Allows one code to be accepted twice inside its window."),
    Field::new(
        "HOTP initial counter",
        "/otpPolicyInitialCounter",
        FieldKind::Integer,
    ),
    Field::new(
        "HOTP look-ahead window",
        "/otpPolicyLookAheadWindow",
        FieldKind::Integer,
    ),
    Field::new(
        "Device code lifespan (s)",
        "/oauth2DeviceCodeLifespan",
        FieldKind::Seconds,
    )
    .help("OAuth 2.0 device authorization grant."),
    Field::new(
        "Device polling interval (s)",
        "/oauth2DevicePollingInterval",
        FieldKind::Integer,
    ),
];
