//! Creates one object of every creatable resource against a real Keycloak.
//!
//! A generic "name plus enabled" body is wrong for most of Keycloak's representations, and
//! the only way to know a template is right is to have the server accept it. Run with
//! `KCT_TEST_SERVER` set; see `live.rs` for the container command.

mod common;

use common::client;
use kct_core::ports::{AdminApi, Context};
use kct_core::{Query, resources};
use serde_json::{Value, json};

/// Parents that nested resources need, created once and reused.
struct Fixtures {
    client: String,
    scope: String,
    group: String,
    idp: String,
    component: String,
}

async fn fixtures(kc: &kct_client::KeycloakClient, realm: &Context) -> Fixtures {
    let make = |id: &'static str, body: Value| async move {
        let resource = resources::by_id(id).unwrap();
        kc.create(resource, realm, body).await.ok().flatten()
    };

    Fixtures {
        client: make(
            "clients",
            json!({"clientId": "kct-fixture", "enabled": true, "protocol": "openid-connect"}),
        )
        .await
        .unwrap_or_default(),
        scope: make(
            "client-scopes",
            json!({"name": "kct-fixture", "protocol": "openid-connect"}),
        )
        .await
        .unwrap_or_default(),
        group: make("groups", json!({"name": "kct-fixture"}))
            .await
            .unwrap_or_default(),
        idp: {
            let resource = resources::by_id("identity-providers").unwrap();
            let body = resource.draft_new("kct-fixture-idp", &realm.params);
            kc.create(resource, realm, body).await.ok();
            "kct-fixture-idp".to_owned()
        },
        component: {
            let resource = resources::by_id("user-federation").unwrap();
            let body = resource.draft_new("kct-fixture-ldap", &realm.params);
            kc.create(resource, realm, body)
                .await
                .ok()
                .flatten()
                .unwrap_or_default()
        },
    }
}

#[tokio::test]
async fn every_creatable_resource_accepts_the_catalogs_template() {
    let Some(kc) = client() else {
        eprintln!("skipped: set KCT_TEST_SERVER to run");
        return;
    };
    let realm = Context::realm(std::env::var("KCT_TEST_REALM").unwrap_or_else(|_| "master".into()));
    let fixtures = fixtures(&kc, &realm).await;

    let mut failures: Vec<String> = Vec::new();
    let mut created = 0;

    for resource in resources::CATALOG {
        if !resource.ops.has(kct_core::catalog::Ops::CREATE) || resource.new_object.is_none() {
            continue;
        }
        // Organizations need a realm-level opt-in; authz needs a resource server.
        if resource.id.starts_with("org") || resource.id.contains("authz") {
            continue;
        }

        let context = match resource.collection_slots().as_slice() {
            [] => realm.clone(),
            ["client"] => realm.clone().with("client", fixtures.client.clone()),
            ["scope"] => realm.clone().with("scope", fixtures.scope.clone()),
            ["group"] => realm.clone().with("group", fixtures.group.clone()),
            ["alias"] => realm.clone().with("alias", fixtures.idp.clone()),
            ["component"] => realm.clone().with("component", fixtures.component.clone()),
            other => {
                failures.push(format!("{}: unhandled parent slots {other:?}", resource.id));
                continue;
            }
        };

        let name = format!("kct-probe-{}", resource.id);
        let body = resource.draft_new(&name, &context.params);

        match kc.create(resource, &context, body.clone()).await {
            Ok(id) => {
                created += 1;
                // Clean up; a failure to delete is not what this test is about.
                let id = id.unwrap_or_else(|| name.clone());
                let _ = kc.remove(resource, &context, &id).await;
            }
            Err(error) => failures.push(format!(
                "{:<26} {error}\n    sent: {}",
                resource.id,
                kct_core::json::compact(&body)
            )),
        }
    }

    // Tidy the fixtures up regardless of the outcome.
    for (id, key) in [
        ("clients", &fixtures.client),
        ("client-scopes", &fixtures.scope),
        ("groups", &fixtures.group),
        ("user-federation", &fixtures.component),
    ] {
        if !key.is_empty() {
            let _ = kc.remove(resources::by_id(id).unwrap(), &realm, key).await;
        }
    }
    let _ = kc
        .remove(
            resources::by_id("identity-providers").unwrap(),
            &realm,
            &fixtures.idp,
        )
        .await;

    assert!(
        created >= 8,
        "expected to exercise most creatable resources, did {created}"
    );
    assert!(
        failures.is_empty(),
        "{} of {} creations were rejected:\n{}",
        failures.len(),
        created + failures.len(),
        failures.join("\n")
    );
}

#[tokio::test]
async fn a_created_object_comes_back_with_what_was_asked_for() {
    let Some(kc) = client() else { return };
    let realm = Context::realm(std::env::var("KCT_TEST_REALM").unwrap_or_else(|_| "master".into()));
    let federation = resources::by_id("user-federation").unwrap();

    let body = federation.draft_new("kct-roundtrip", &realm.params);
    let id = kc
        .create(federation, &realm, body)
        .await
        .expect("an LDAP provider template must be valid")
        .expect("Keycloak reports the new component id");

    let fetched = kc.fetch(federation, &realm, &id).await.expect("fetch");
    assert_eq!(fetched["providerId"], json!("ldap"));
    assert_eq!(fetched["config"]["editMode"], json!(["READ_ONLY"]));
    assert_eq!(
        fetched["config"]["enabled"],
        json!(["false"]),
        "a new provider starts disabled so a blank URL cannot break user lookups"
    );

    // And its mappers, whose parent must be threaded through.
    let mappers = resources::by_id("federation-mappers").unwrap();
    let nested = realm.clone().with("component", id.clone());
    let mapper_body = mappers.draft_new("kct-mapper", &nested.params);
    assert_eq!(
        mapper_body["parentId"],
        json!(id),
        "a mapper must name its parent"
    );

    kc.create(mappers, &nested, mapper_body)
        .await
        .expect("mapper template must be valid");

    let listed = kc
        .list(mappers, &nested, &Query::new())
        .await
        .expect("list");
    assert!(
        listed
            .items
            .iter()
            .any(|m| m["name"] == json!("kct-mapper"))
    );

    let _ = kc.remove(federation, &realm, &id).await;
}
