//! The SCIM failure ladder the client-side probe explains, pinned against a real server.
//!
//! Skipped without `KCT_TEST_SERVER`, and on servers where `scim-api` is off (26.7 and
//! older keep it as a disabled preview).

mod common;

use kct_core::ports::{Expect, Gateway, Rest};
use kct_core::{ApiPath, Error, Method, Request, Secret};
use serde_json::{Value, json};

const REALM: &str = "kct-scim-live";

async fn admin(gateway: &impl Gateway, method: Method, path: &str, body: Option<Value>) {
    let mut request = Request::new(method, ApiPath::new(path)).expecting(Expect::Nothing);
    if let Some(body) = body {
        request = request.with_body(body);
    }
    gateway.send(request).await.expect(path);
}

async fn client_token(gateway: &impl Gateway) -> Secret {
    let request = Request::new(
        Method::Post,
        ApiPath::new(format!("/realms/{REALM}/protocol/openid-connect/token")),
    )
    .with_form(vec![
        ("grant_type".into(), "client_credentials".into()),
        ("client_id".into(), "scim-sync".into()),
        ("client_secret".into(), "scim-sync-secret".into()),
    ])
    .anonymous();
    let body = gateway.send(request).await.expect("token").body;
    Secret::new(body["access_token"].as_str().expect("access_token"))
}

async fn scim_status(gateway: &impl Gateway, token: Secret) -> u16 {
    let path = ApiPath::new(format!("/realms/{REALM}/scim/v2/Users"));
    match gateway.send(Request::get(path).with_bearer(token)).await {
        Ok(_) => 200,
        Err(Error::Api { status, .. }) => status,
        Err(other) => panic!("unexpected failure: {other}"),
    }
}

#[tokio::test]
async fn each_missing_piece_of_scim_setup_has_its_own_status() {
    let Some(gateway) = common::client() else {
        eprintln!("skipped: set KCT_TEST_SERVER to run");
        return;
    };
    let server = std::env::var("KCT_TEST_SERVER").unwrap();
    let _ = gateway
        .send(Request::new(
            Method::Delete,
            ApiPath::new(format!("/admin/realms/{REALM}")),
        ))
        .await;

    admin(
        &gateway,
        Method::Post,
        "/admin/realms",
        Some(json!({"realm": REALM, "enabled": true})),
    )
    .await;
    let clients = format!("/admin/realms/{REALM}/clients");
    let client = json!({"clientId": "scim-sync", "publicClient": false,
        "serviceAccountsEnabled": true, "secret": "scim-sync-secret"});
    admin(&gateway, Method::Post, &clients, Some(client)).await;

    assert_eq!(
        scim_status(&gateway, client_token(&gateway).await).await,
        404,
        "realm switch off"
    );

    let realm = format!("/admin/realms/{REALM}");
    admin(
        &gateway,
        Method::Put,
        &realm,
        Some(json!({"scimApiEnabled": true})),
    )
    .await;
    let without_audience = scim_status(&gateway, client_token(&gateway).await).await;
    if without_audience == 404 {
        eprintln!("skipped: the scim-api feature is off on this server");
        admin(&gateway, Method::Delete, &realm, None).await;
        return;
    }
    assert_eq!(without_audience, 401, "token without the SCIM audience");

    let found = gateway
        .get_json(
            ApiPath::new(clients.clone()),
            vec![("clientId".into(), "scim-sync".into())],
        )
        .await
        .unwrap();
    let id = found[0]["id"].as_str().unwrap();
    let mapper = json!({"name": "scim-audience", "protocol": "openid-connect",
        "protocolMapper": "oidc-audience-mapper",
        "config": {"included.custom.audience": format!("{server}/realms/{REALM}/scim/v2"),
                   "access.token.claim": "true"}});
    admin(
        &gateway,
        Method::Post,
        &format!("{clients}/{id}/protocol-mappers/models"),
        Some(mapper),
    )
    .await;
    assert_eq!(
        scim_status(&gateway, client_token(&gateway).await).await,
        403,
        "no admin roles"
    );

    admin(&gateway, Method::Delete, &realm, None).await;
}
