//! Vocabularies Keycloak documents but does not publish through `serverinfo`.
//!
//! Every list here is a fallback: when the live server can answer instead, it wins, because a
//! static list goes stale on the next release and a wrong suggestion costs more than none.

pub const SIGNATURE_ALGORITHMS: &[&str] = &[
    "RS256", "RS384", "RS512", "PS256", "PS384", "PS512", "ES256", "ES384", "ES512", "HS256",
    "HS384", "HS512", "EdDSA",
];

pub const REQUIRED_ACTIONS: &[&str] = &[
    "VERIFY_EMAIL",
    "UPDATE_PROFILE",
    "UPDATE_PASSWORD",
    "CONFIGURE_TOTP",
    "CONFIGURE_RECOVERY_AUTHN_CODES",
    "TERMS_AND_CONDITIONS",
    "delete_account",
    "VERIFY_PROFILE",
    "UPDATE_EMAIL",
];

pub const LOCALES: &[&str] = &[
    "en", "de", "fr", "es", "it", "nl", "pl", "pt-BR", "cs", "sk", "ru", "uk", "ja", "zh-CN",
];

pub const PROTOCOLS: &[&str] = &["openid-connect", "saml"];
pub const OTP_TYPES: &[&str] = &["totp", "hotp"];
pub const OTP_ALGORITHMS: &[&str] = &["HmacSHA1", "HmacSHA256", "HmacSHA512"];
pub const OTP_DIGITS: &[&str] = &["6", "8"];
pub const SSL_REQUIRED: &[&str] = &["all", "external", "none"];
pub const ACCESS_TYPES: &[&str] = &["PUBLIC", "CONFIDENTIAL", "BEARER-ONLY"];

/// Flow-binding fields all take the alias of a top-level flow.
pub const FLOW_BINDINGS: &[&str] = &[
    "/browserFlow",
    "/registrationFlow",
    "/directGrantFlow",
    "/resetCredentialsFlow",
    "/clientAuthenticationFlow",
    "/dockerAuthenticationFlow",
    "/firstBrokerLoginFlow",
    "/authenticationFlowBindingOverrides/browser",
    "/authenticationFlowBindingOverrides/direct_grant",
    // Identity providers spell the same thing with an `Alias` suffix.
    "/firstBrokerLoginFlowAlias",
    "/postBrokerLoginFlowAlias",
];
