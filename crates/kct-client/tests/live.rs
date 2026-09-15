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
use kct_core::ports::{AdminApi, Context, Rest};
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
async fn authenticates_and_reports_what_the_server_supports() {
    live!(client);

    client.authenticate().await.expect("password grant works");
    let profile = client.server_profile("master").await.expect("serverinfo");

    assert!(!profile.version.is_unknown(), "version must be readable");
    assert!(profile.version.at_least(kct_core::Version::new(25, 0, 0)));
    assert!(
        !profile.features.is_empty(),
        "a complete profile lists feature flags"
    );
    assert!(
        profile.has_provider("authenticator", "auth-otp-form"),
        "built-in authenticators should be discoverable"
    );
}

#[tokio::test]
async fn lists_every_top_level_screen_without_error() {
    live!(client);

    let context = Context::realm("master");
    let mut checked = 0;

    for resource in resources::top_level() {
        let profile = client.server_profile("master").await.expect("serverinfo");
        if !profile.evaluate(&resource.requirement).is_permitted() {
            continue;
        }
        // Realms are server-scoped, and events need to be enabled first.
        if matches!(
            resource.id,
            "realms" | "server-features" | "server-providers" | "server-info"
        ) {
            continue;
        }

        let page = match client.list(resource, &context, &Query::page(0, 5)).await {
            Ok(page) => page,
            // A feature can be supported by the server and still switched off for the realm;
            // that is configuration, not a broken endpoint.
            Err(error) if error.to_string().contains("not enabled") => continue,
            Err(error) => panic!("listing {} failed: {error}", resource.id),
        };

        for row in page.items.iter() {
            assert!(
                row.is_object(),
                "{} produced a non-object row: {row}",
                resource.id
            );
        }
        checked += 1;
    }

    assert!(
        checked > 10,
        "expected to exercise most screens, did {checked}"
    );
}

#[tokio::test]
async fn a_full_user_lifecycle_never_loses_a_field() {
    live!(client);

    let users = resources::by_id("users").unwrap();
    let context = Context::realm("master");
    let username = format!("kct-live-{}", std::process::id());

    let created = client
        .create(
            users,
            &context,
            json!({
                "username": username,
                "enabled": true,
                "email": "kct@example.test",
                "attributes": {"department": ["platform"], "costCentre": ["42"]}
            }),
        )
        .await
        .expect("create");
    let id = created.expect("Keycloak reports the new id in Location");

    let fetched = client.fetch(users, &context, &id).await.expect("fetch");
    assert_eq!(fetched["username"], json!(username));

    // Keycloak's declarative user profile discards attributes it does not declare, and says
    // nothing about it: the create returned 201 and the attribute is simply absent. The UI
    // warns about this before a save; here we pin the server behaviour the warning exists for.
    let profile = client
        .get_json(
            kct_core::path::endpoints::USER_PROFILE
                .resolve(&kct_core::PathParams::realm("master"))
                .unwrap(),
            Vec::new(),
        )
        .await
        .expect("user profile");
    let unmanaged_allowed = profile
        .get("unmanagedAttributePolicy")
        .is_some_and(|p| !p.is_null());

    assert_eq!(
        fetched["attributes"]["department"].is_null(),
        !unmanaged_allowed,
        "attribute survival must follow the realm's unmanaged-attribute policy"
    );

    // Change exactly one field the way the UI does, then check nothing else moved.
    client
        .save(users, &context, &id, &fetched, json!({"enabled": false}))
        .await
        .expect("save");

    let after = client.fetch(users, &context, &id).await.expect("re-fetch");
    assert_eq!(after["enabled"], json!(false));
    assert_eq!(
        after["email"],
        json!("kct@example.test"),
        "a narrow edit must not drop fields the patch never mentioned"
    );
    assert_eq!(after["username"], json!(username));

    client.remove(users, &context, &id).await.expect("delete");
    assert!(
        client.fetch(users, &context, &id).await.is_err(),
        "the user should be gone"
    );
}
