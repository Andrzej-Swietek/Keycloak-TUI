use kct_core::ports::{Gateway, Request};
use kct_core::{ApiPath, Error, Method, Secret};

use crate::config::{Connection, Credentials};

use super::KeycloakClient;
use super::errors::{MAX_ERROR_BODY, api_error};

fn connection(dry_run: bool) -> Connection {
    Connection::new(
        "https://kc.test",
        "master",
        "acme",
        Credentials::ClientCredentials {
            client_id: "admin-cli".into(),
            client_secret: Secret::new("s"),
        },
    )
    .unwrap()
    .with_dry_run(dry_run)
}

#[test]
fn the_client_builds_and_exposes_its_base_url() {
    let client = KeycloakClient::connect(connection(false)).unwrap();
    assert_eq!(client.base_url(), "https://kc.test");
    assert_eq!(client.connection().realm, "acme");
}

#[tokio::test]
async fn dry_run_blocks_writes_but_not_reads() {
    let client = KeycloakClient::connect(connection(true)).unwrap();

    let write = client
        .send(Request::new(
            Method::Delete,
            ApiPath::new("/admin/realms/x"),
        ))
        .await;
    let message = write.unwrap_err().to_string();
    assert!(message.contains("dry-run"), "{message}");

    // A read is not blocked; it fails later, on the network, which is a different error.
    let read = client
        .send(Request::get(ApiPath::new("/admin/realms")))
        .await;
    assert!(!read.unwrap_err().to_string().contains("dry-run"));
}

#[test]
fn keycloaks_own_error_message_survives_into_the_error() {
    let request = Request::new(Method::Post, ApiPath::new("/admin/realms/r/users"));
    let error = api_error(
        &request,
        409,
        r#"{"errorMessage":"User exists with same username"}"#,
    );
    assert!(error.to_string().contains("User exists with same username"));
    assert_eq!(error.status(), Some(409));
    assert!(error.hint().unwrap().contains("already exists"));
}

#[test]
fn an_unparseable_error_body_still_yields_something_useful() {
    let request = Request::get(ApiPath::new("/admin/realms"));
    let error = api_error(&request, 502, "<html>gateway error</html>");
    assert!(error.to_string().contains("HTTP 502"));
    assert!(error.is_retryable());
}

#[test]
fn oversized_error_bodies_are_truncated_before_they_reach_a_log() {
    let request = Request::get(ApiPath::new("/x"));
    let error = api_error(&request, 500, &"A".repeat(50_000));
    let Error::Api { detail, .. } = error else {
        panic!("expected an API error")
    };
    assert_eq!(detail.unwrap().len(), MAX_ERROR_BODY);
}

#[test]
fn reads_are_retried_and_writes_are_not() {
    // Encoded as a property of Method so it cannot drift from the retry loop.
    assert!(!Method::Get.is_mutating());
    assert!(Method::Post.is_mutating());
    assert!(Method::Put.is_mutating());
    assert!(Method::Delete.is_mutating());
}
