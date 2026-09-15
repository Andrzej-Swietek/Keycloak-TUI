use super::*;

#[test]
fn resolves_multi_slot_templates() {
    let params = PathParams::realm("avsystem")
        .with("ownerKind", "users")
        .with("owner", "42")
        .with("client", "abc");

    assert_eq!(
        endpoints::ROLE_MAPPINGS_CLIENT
            .resolve(&params)
            .unwrap()
            .as_str(),
        "/admin/realms/avsystem/users/42/role-mappings/clients/abc"
    );
}

#[test]
fn identifiers_that_would_break_the_url_are_encoded() {
    let params = PathParams::realm("my realm").with("user", "cn=admin,ou=people/x");

    assert_eq!(
        endpoints::USER.resolve(&params).unwrap().as_str(),
        "/admin/realms/my%20realm/users/cn%3Dadmin%2Cou%3Dpeople%2Fx"
    );
}

#[test]
fn a_missing_slot_is_a_readable_error_not_a_panic() {
    let message = endpoints::USER
        .resolve(&PathParams::realm("master"))
        .unwrap_err()
        .to_string();

    assert!(message.contains("needs 'user'"), "{message}");
    assert!(message.contains("realm"), "{message}");
}

#[test]
fn templates_declare_their_own_slots() {
    assert_eq!(
        endpoints::ROLE_MAPPINGS_CLIENT.slots(),
        vec!["realm", "ownerKind", "owner", "client"]
    );
    assert!(endpoints::SERVER_INFO.slots().is_empty());
}

#[test]
fn paths_compose_without_double_slashes() {
    let base = endpoints::USER
        .resolve(&PathParams::realm("r").with("user", "u"))
        .unwrap();

    assert_eq!(
        base.sub("role-mappings/realm").as_str(),
        "/admin/realms/r/users/u/role-mappings/realm"
    );
    assert_eq!(base.child("a b").as_str(), "/admin/realms/r/users/u/a%20b");
}
