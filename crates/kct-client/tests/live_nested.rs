//! Integration tests against a real Keycloak.
//!
//! Skipped unless `KCT_TEST_SERVER` points at one, so `cargo test` stays offline by default:
//!
//! ```text
//! docker run -d -p 18080:8080 -e KC_BOOTSTRAP_ADMIN_USERNAME=admin \
//!   -e KC_BOOTSTRAP_ADMIN_PASSWORD=admin quay.io/keycloak/keycloak:26.4 start-dev
//! KCT_TEST_SERVER=http://localhost:18080 cargo test -p kct-client --test live
//! ```

use kct_client::{Connection, Credentials, KeycloakClient};
use kct_core::ports::{AdminApi, Context};
use kct_core::{Query, Secret, resources};
use serde_json::json;

fn client() -> Option<KeycloakClient> {
    let server = std::env::var("KCT_TEST_SERVER").ok()?;
    let connection = Connection::new(
        server,
        "master",
        "master",
        Credentials::Password {
            client_id: "admin-cli".into(),
            username: std::env::var("KCT_TEST_USER").unwrap_or_else(|_| "admin".into()),
            password: Secret::new(
                std::env::var("KCT_TEST_PASSWORD").unwrap_or_else(|_| "admin".into()),
            ),
            client_secret: None,
            otp: None,
        },
    )
    .expect("valid connection");

    Some(KeycloakClient::connect(connection).expect("client builds"))
}

macro_rules! live {
    ($client:ident) => {
        let Some($client) = client() else {
            eprintln!("skipped: set KCT_TEST_SERVER to run");
            return;
        };
    };
}

#[tokio::test]
async fn nested_collections_resolve_against_a_real_parent() {
    live!(client);

    let clients = resources::by_id("clients").unwrap();
    let context = Context::realm("master");

    let page = client
        .list(clients, &context, &Query::page(0, 50))
        .await
        .expect("list clients");
    let account = page
        .items
        .iter()
        .find(|row| row["clientId"] == json!("account"))
        .expect("the built-in `account` client exists");
    let id = account["id"].as_str().unwrap();

    let nested = Context::realm("master").with("client", id);
    for child in ["client-roles", "client-mappers", "client-default-scopes"] {
        let resource = resources::by_id(child).unwrap();
        client
            .list(resource, &nested, &Query::page(0, 20))
            .await
            .unwrap_or_else(|error| panic!("{child} failed: {error}"));
    }
}

#[tokio::test]
async fn realm_keys_and_serverinfo_shapes_match_the_catalog() {
    live!(client);

    let keys = resources::by_id("realm-keys").unwrap();
    let page = client
        .list(keys, &Context::realm("master"), &Query::new())
        .await
        .expect("keys");

    assert!(!page.is_empty(), "a realm always has signing keys");
    assert!(
        page.items
            .iter()
            .any(|key| key["algorithm"] == json!("RS256"))
    );

    let features = resources::by_id("server-features").unwrap();
    let page = client
        .list(features, &Context::server(), &Query::new())
        .await
        .expect("features");

    assert!(!page.is_empty());
    assert!(page.items.iter().all(|flag| flag["name"].is_string()));
}
