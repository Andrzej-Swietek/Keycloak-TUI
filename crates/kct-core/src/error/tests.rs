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

#[test]
fn a_policy_type_that_needs_more_than_a_name_says_so() {
    // Keycloak answers 500 with "consult the server log" for a group or client policy created
    // with only a name; the generic 500 hint points at user federation, which is wrong here.
    let error = Error::Api {
        status: 500,
        method: Method::Post,
        path: "/admin/realms/acme/clients/c1/authz/resource-server/policy/group".to_owned(),
        message: "unknown_error".to_owned(),
        detail: None,
    };

    let hint = hint::for_error(&error).expect("a 500 always carries a hint");
    assert!(hint.contains("policy type"), "{hint}");
    assert!(!hint.contains("federation"), "{hint}");
}

#[test]
fn a_masked_idp_secret_sent_to_a_new_endpoint_asks_for_the_secret() {
    // Verified against Keycloak 26.8.1: changing `tokenUrl` while the body still carries the
    // masked `**********` secret is refused with this exact message.
    let error = Error::Api {
        status: 400,
        method: Method::Put,
        path: "/admin/realms/acme/identity-provider/instances/oidc1".to_owned(),
        message: "Client secret must be re-entered when the token URL, client ID, \
                  authentication method, or related destination settings are changed"
            .to_owned(),
        detail: None,
    };

    let hint = hint::for_error(&error).expect("a 400 always carries a hint");
    assert!(hint.contains("Type the secret again"), "{hint}");
}

#[test]
fn a_forbidden_client_secret_names_manage_clients() {
    // Keycloak 26.8.0 stopped showing secrets to `view-clients`; the endpoint answers 403.
    let error = Error::Api {
        status: 403,
        method: Method::Get,
        path: "/admin/realms/acme/clients/c1/client-secret".to_owned(),
        message: "HTTP 403 Forbidden".to_owned(),
        detail: None,
    };

    assert!(hint::for_error(&error).unwrap().contains("manage-clients"));
}
