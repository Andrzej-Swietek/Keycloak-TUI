use super::*;
use base64::Engine;
use serde_json::json;

fn encode(header: &Value, payload: &Value) -> String {
    let part = |value: &Value| {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(value.to_string().as_bytes())
    };
    format!("{}.{}.c2ln", part(header), part(payload))
}

#[test]
fn decodes_a_keycloak_access_token() {
    let raw = encode(
        &json!({"alg": "RS256", "typ": "JWT", "kid": "abc123"}),
        &json!({
            "iss": "https://kc/realms/avsystem",
            "sub": "u1",
            "azp": "coiote-ae",
            "aud": ["account", "coiote-ae"],
            "scope": "openid profile email",
            "preferred_username": "alice",
            "realm_access": {"roles": ["deploy-approver", "offline_access"]},
            "exp": chrono::Utc::now().timestamp() + 300
        }),
    );

    let token = Token::decode(&raw).unwrap();
    assert_eq!(token.signed_with(), "RS256 (kid abc123)");

    let claims = token.claims();
    assert_eq!(claims.preferred_username.as_deref(), Some("alice"));
    assert_eq!(claims.authorized_party.as_deref(), Some("coiote-ae"));
    assert_eq!(claims.audience, vec!["account", "coiote-ae"]);
    assert_eq!(claims.scope, vec!["openid", "profile", "email"]);
    assert_eq!(claims.realm_roles.len(), 2);
    assert!(matches!(claims.expiry, Expiry::Valid(_)));
}

#[test]
fn a_single_audience_is_a_string_not_an_array() {
    // Both shapes are legal and Keycloak emits both depending on the client's mappers.
    let raw = encode(&json!({"alg": "RS256"}), &json!({"aud": "account"}));
    assert_eq!(
        Token::decode(&raw).unwrap().claims().audience,
        vec!["account"]
    );
}

#[test]
fn an_expired_token_says_how_long_ago() {
    let raw = encode(
        &json!({"alg": "RS256"}),
        &json!({"exp": chrono::Utc::now().timestamp() - 90}),
    );

    let expiry = Token::decode(&raw).unwrap().claims().expiry;
    assert!(expiry.is_expired());
    assert!(expiry.describe().starts_with("EXPIRED 1m"));
}

#[test]
fn an_opaque_token_is_reported_not_guessed_at() {
    // Keycloak refresh tokens are often opaque; pretending otherwise would be worse.
    let error = Token::decode("not-a-jwt").unwrap_err().to_string();
    assert!(error.contains("3 dot-separated segments"), "{error}");
}

#[test]
fn a_token_with_no_expiry_claim_is_not_reported_as_expired() {
    let raw = encode(&json!({"alg": "none"}), &json!({"sub": "u1"}));
    let expiry = Token::decode(&raw).unwrap().claims().expiry;

    assert_eq!(expiry, Expiry::Unknown);
    assert!(!expiry.is_expired());
}
