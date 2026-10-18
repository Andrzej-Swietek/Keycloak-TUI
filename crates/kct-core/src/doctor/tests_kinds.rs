//! Field controls against the shape of the data behind them.

use super::{Finding, compare_kinds};
use crate::resources;
use serde_json::json;

#[test]
fn a_flat_map_behind_a_raw_json_field_is_reported() {
    let mappers = resources::by_id("client-mappers").unwrap();
    let sample = json!({
        "name": "email",
        "config": {"claim.name": "email", "jsonType.label": "String"},
    });

    let findings = compare_kinds(mappers, std::slice::from_ref(&sample));
    assert!(
        findings.is_empty(),
        "config is declared StringMap now: {findings:?}"
    );
}

#[test]
fn a_boolean_stored_as_a_string_is_reported() {
    let realm = resources::by_id("realm-settings").unwrap();
    let sample = json!({"smtpServer": {"auth": "true", "ssl": "false"}});

    let findings = compare_kinds(realm, std::slice::from_ref(&sample));
    assert!(
        findings.is_empty(),
        "the SMTP switches are stringly Bools now: {findings:?}"
    );
}

#[test]
fn a_component_config_list_is_not_a_mismatch() {
    // `{"enabled": ["true"]}` is what a component really sends; the field's encoding says
    // so, and flagging it would bury the real findings in noise.
    let federation = resources::by_id("user-federation").unwrap();
    let sample = json!({"config": {"enabled": ["true"], "usersDn": ["ou=people"]}});

    assert_eq!(
        compare_kinds(federation, std::slice::from_ref(&sample)),
        Vec::<Finding>::new()
    );
}

#[test]
fn an_unmatched_control_is_still_caught() {
    let users = resources::by_id("users").unwrap();
    let sample = json!({"username": {"a": "b"}});

    let findings = compare_kinds(users, std::slice::from_ref(&sample));
    assert!(
        findings.iter().any(|f| f.detail.contains("StringMap")),
        "a map behind a text box should be reported: {findings:?}"
    );
}
