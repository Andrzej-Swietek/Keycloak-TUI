use super::*;

#[test]
fn transport_failures_are_worth_retrying_but_forbidden_is_not() {
    let transport = Error::Transport {
        url: "https://kc".into(),
        reason: "connection reset".into(),
    };
    assert!(transport.is_retryable());

    let forbidden = Error::Api {
        status: 403,
        method: Method::Get,
        path: "/admin/realms".into(),
        message: "Forbidden".into(),
        detail: None,
    };
    assert!(!forbidden.is_retryable());
    assert!(forbidden.hint().is_some());
}

fn forbidden(path: &str) -> Error {
    Error::Api {
        status: 403,
        method: Method::Get,
        path: path.into(),
        message: "Forbidden".into(),
        detail: None,
    }
}

#[test]
fn a_403_names_the_role_the_account_is_missing() {
    // Verified against Keycloak 26.7.3: a delegated admin with only `view-realm` and
    // `view-users` is refused on clients, and "403" alone leaves them guessing.
    let clients = forbidden("/admin/realms/tenant/clients");
    assert!(
        clients.hint().unwrap().contains("view-clients"),
        "{}",
        clients.hint().unwrap()
    );

    assert!(
        forbidden("/admin/realms/t/users")
            .hint()
            .unwrap()
            .contains("view-users")
    );
    assert!(
        forbidden("/admin/realms/t/components")
            .hint()
            .unwrap()
            .contains("view-realm")
    );
    assert!(
        forbidden("/admin/realms/t/clients/x/authz/resource-server/policy")
            .hint()
            .unwrap()
            .contains("authorization")
    );
}

#[test]
fn an_unrecognised_path_still_gets_useful_advice() {
    let hint = forbidden("/admin/serverinfo").hint().unwrap();
    assert!(hint.contains("realm-management"));
}

#[test]
fn only_get_is_treated_as_non_mutating() {
    assert!(!Method::Get.is_mutating());
    for method in [Method::Post, Method::Put, Method::Delete, Method::Patch] {
        assert!(method.is_mutating(), "{method} must count as a write");
    }
}
