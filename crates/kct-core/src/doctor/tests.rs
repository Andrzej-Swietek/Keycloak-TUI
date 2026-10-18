use super::*;
use crate::capability::ServerProfile;
use crate::resources;
use serde_json::json;

#[test]
fn a_field_no_object_carries_is_flagged_once_there_are_enough_objects() {
    let users = resources::by_id("users").unwrap();
    let sample = json!({
        "id": "u1",
        "username": "a",
        "email": "a@b.test",
        "firstName": "A",
        "lastName": "B",
        "enabled": true,
        "requiredActions": [],
        "federationLink": null,
        "createdTimestamp": 1,
        "attributes": {}
    });

    // One object proves nothing: Keycloak omits an unset optional field.
    assert!(compare_fields(users, std::slice::from_ref(&sample)).is_empty());

    let many = vec![sample.clone(), sample.clone(), sample];
    let findings = compare_fields(users, &many);

    assert!(
        findings
            .iter()
            .any(|finding| finding.detail.contains("/emailVerified")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding.severity != Severity::Broken),
        "an unset field is not a breakage"
    );
}

#[test]
fn a_new_server_field_is_reported_as_missing_not_broken() {
    let users = resources::by_id("users").unwrap();
    let sample = json!({
        "id": "u1",
        "username": "alice",
        "email": "a@b.c",
        "emailVerified": true,
        "firstName": "A",
        "lastName": "B",
        "enabled": true,
        "requiredActions": [],
        "federationLink": null,
        "createdTimestamp": 1,
        "attributes": {},
        "somethingKeycloak27Added": "x"
    });

    let findings = compare_fields(users, &[sample]);
    assert!(findings.iter().any(|finding| {
        finding.severity == Severity::Missing && finding.detail.contains("somethingKeycloak27Added")
    }));
    assert!(
        findings
            .iter()
            .all(|finding| finding.severity != Severity::Broken),
        "a new field is not a breakage: {findings:?}"
    );
}

#[test]
fn component_forms_are_checked_against_the_servers_own_property_list() {
    let federation = resources::by_id("user-federation").unwrap();
    let profile = ServerProfile::from_server_info(json!({
        "systemInfo": {"version": "26.8.1"},
        "componentTypes": {
            "org.keycloak.storage.UserStorageProvider": [{
                "id": "ldap",
                "properties": [
                    {"name": "connectionUrl"},
                    {"name": "bindDn"},
                    {"name": "aBrandNewSetting"}
                ]
            }]
        }
    }));

    let findings = compare_component(
        federation,
        &profile,
        "org.keycloak.storage.UserStorageProvider",
        "ldap",
    );

    assert!(
        findings
            .iter()
            .any(|f| f.detail.contains("aBrandNewSetting") && f.severity == Severity::Missing),
        "a newly accepted setting should be reported"
    );
    assert!(
        findings
            .iter()
            .any(|f| f.detail.contains("usersDn") && f.severity == Severity::Note),
        "an unadvertised key is reported, but the list is not exhaustive: {findings:?}"
    );
}

#[test]
fn a_gate_on_a_feature_the_server_never_heard_of_is_broken() {
    let profile = ServerProfile::from_server_info(json!({
        "systemInfo": {"version": "26.8.1"},
        "features": [{"name": "organization", "type": "DEFAULT", "enabled": true}]
    }));

    let findings = check_feature_gates(&profile);
    // `authorization` is gated on by the authz screens and is absent from this profile.
    assert!(
        findings
            .iter()
            .any(|finding| finding.detail.contains("authorization"))
    );
}

#[test]
fn an_incomplete_profile_reports_nothing_rather_than_everything() {
    let degraded = ServerProfile::from_discovery(crate::Version::new(26, 7, 3));
    assert!(check_feature_gates(&degraded).is_empty());
}

#[test]
fn a_report_sorts_the_worst_first_and_knows_when_it_is_clean() {
    let report = Report::default()
        .with([
            Finding::note("a", "just so you know"),
            Finding::broken("b", "gone"),
            Finding::missing("c", "new"),
        ])
        .sorted();

    assert_eq!(report.findings[0].severity, Severity::Broken);
    assert!(!report.is_healthy());
    assert_eq!(report.count(Severity::Missing), 1);

    assert!(
        Report::default()
            .with([Finding::note("a", "x")])
            .is_healthy()
    );
}
