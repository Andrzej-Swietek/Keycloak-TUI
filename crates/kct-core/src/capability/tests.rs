use super::*;
use serde_json::json;

fn profile() -> ServerProfile {
    ServerProfile::from_server_info(json!({
        "systemInfo": {"version": "26.6.1"},
        "features": [
            {"name": "organization", "type": "DEFAULT", "enabled": true},
            {"name": "scim", "type": "PREVIEW", "enabled": false},
            {"name": "token-exchange", "type": "PREVIEW", "enabled": true}
        ],
        "providers": {"policy": {"providers": {"group-hierarchy": {}, "role": {}}}},
        "componentTypes": {
            "org.keycloak.storage.UserStorageProvider": [{"id": "ldap"}, {"id": "kerberos"}]
        },
        "protocolMapperTypes": {
            "openid-connect": [{"id": "oidc-usermodel-attribute-mapper"}]
        }
    }))
}

#[test]
fn parses_every_section_of_serverinfo() {
    let profile = profile();
    assert_eq!(profile.version, Version::new(26, 6, 1));
    assert_eq!(profile.source, ProfileSource::ServerInfo);
    assert!(profile.has_provider("policy", "group-hierarchy"));
    assert!(!profile.has_provider("policy", "nonexistent"));
    assert_eq!(
        profile.component_types["org.keycloak.storage.UserStorageProvider"].len(),
        2
    );
    assert_eq!(profile.protocol_mapper_types["openid-connect"].len(), 1);
}

#[test]
fn disabled_and_missing_features_read_differently() {
    let profile = profile();
    assert_eq!(
        profile.evaluate(&Requirement::Feature(features::ORGANIZATION)),
        Support::Available
    );

    let scim = profile.evaluate(&Requirement::Feature(features::SCIM));
    assert!(!scim.is_permitted());
    assert!(scim.reason().unwrap().contains("--features=scim"));

    let absent = profile.evaluate(&Requirement::Feature("no-such-feature"));
    assert!(matches!(absent, Support::Unavailable(_)));
}

#[test]
fn an_incomplete_profile_never_hides_a_feature() {
    let degraded = ServerProfile::from_discovery(Version::new(26, 6, 1));
    let support = degraded.evaluate(&Requirement::Feature(features::ORGANIZATION));

    assert!(matches!(support, Support::Unknown(_)));
    assert!(support.is_permitted(), "unknown must let the operator try");
}

#[test]
fn unknown_versions_permit_everything_rather_than_blocking_work() {
    let blind = ServerProfile::assumed(Version::UNKNOWN);
    assert!(
        blind
            .evaluate(&Requirement::MinVersion(Version::new(26, 0, 0)))
            .is_permitted()
    );
}

#[test]
fn old_servers_are_told_exactly_what_they_are_missing() {
    let old = ServerProfile::from_discovery(Version::new(25, 0, 6));
    assert_eq!(
        old.evaluate(&Requirement::MinVersion(Version::new(26, 0, 0))),
        Support::Unavailable("needs Keycloak 26.0.0 or newer (this server is 25.0.6)".into())
    );
}

#[test]
fn combinators_pick_the_pessimistic_and_optimistic_answers() {
    static BOTH: &[Requirement] = &[
        Requirement::MinVersion(Version::new(26, 0, 0)),
        Requirement::Feature(features::SCIM),
    ];
    let profile = profile();
    assert!(!profile.evaluate(&Requirement::All(BOTH)).is_permitted());
    assert!(profile.evaluate(&Requirement::Any(BOTH)).is_permitted());
}

#[test]
fn preview_features_in_use_are_reported_for_incident_context() {
    let profile = profile();
    let risky = profile.risky_enabled_features();

    assert_eq!(risky.len(), 1);
    assert_eq!(risky[0].name, "token-exchange");
    assert_eq!(risky[0].kind.badge(), "preview");
}

#[test]
fn a_snapshot_build_is_labelled_as_such() {
    let nightly = ServerProfile::from_server_info(json!({
        "systemInfo": {"version": "999.0.0-SNAPSHOT"}
    }));
    assert!(nightly.version.is_nightly());
    assert_eq!(nightly.version_display(), "999.0.0 (SNAPSHOT)");
}

#[test]
fn a_realm_scoped_admin_keeps_capability_detection_without_the_version() {
    // Verified against Keycloak 26.7.3: with `view-system` removed, `serverinfo` still
    // carries features and providers but omits `systemInfo` entirely.
    let trimmed = ServerProfile::from_server_info(json!({
        "features": [{"name": "organization", "type": "DEFAULT", "enabled": true}],
        "providers": {"policy": {"providers": {"role": {}}}}
    }));

    assert!(trimmed.version.is_unknown());
    assert_eq!(trimmed.version_summary(), "version hidden");
    assert!(trimmed.knows_capabilities(), "features are still usable");
    assert_eq!(
        trimmed.evaluate(&Requirement::Feature(features::ORGANIZATION)),
        Support::Available
    );
}

#[test]
fn an_unknown_version_never_renders_as_a_stutter() {
    let blind = ServerProfile::assumed(Version::UNKNOWN);
    assert_eq!(blind.version_display(), "unknown");
    assert_eq!(blind.version_summary(), "not detected");
}

#[test]
fn a_feature_is_found_however_keycloak_spells_it() {
    // 26.7 reports the enum name; older releases reported the `--features=` spelling. Both
    // must resolve, or every gate quietly turns into "this feature does not exist".
    let modern = ServerProfile::from_server_info(json!({
        "systemInfo": {"version": "26.7.3"},
        "features": [
            {"name": "ORGANIZATION", "type": "DEFAULT", "enabled": true},
            {"name": "CLIENT_POLICIES", "type": "DEFAULT", "enabled": true}
        ]
    }));

    assert!(modern.feature(features::ORGANIZATION).is_some());
    assert_eq!(
        modern.evaluate(&Requirement::Feature(features::CLIENT_POLICIES)),
        Support::Available
    );

    let older = ServerProfile::from_server_info(json!({
        "systemInfo": {"version": "26.0.0"},
        "features": [{"name": "organization", "type": "DEFAULT", "enabled": true}]
    }));
    assert_eq!(
        older.evaluate(&Requirement::Feature(features::ORGANIZATION)),
        Support::Available
    );
}

#[test]
fn a_feature_that_really_is_absent_is_still_reported() {
    let profile = ServerProfile::from_server_info(json!({
        "systemInfo": {"version": "26.7.3"},
        "features": [{"name": "ORGANIZATION", "type": "DEFAULT", "enabled": true}]
    }));
    assert!(profile.feature("no-such-feature").is_none());
}
