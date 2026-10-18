use std::sync::Mutex;

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::error::Error;
use crate::error::Result;
use crate::query::Query;
use crate::resources;

use super::*;

/// A `Gateway` test double. One method, and the whole `AdminApi` surface works against
/// it — which is the point of the port being this narrow.
#[derive(Debug, Default)]
struct Recorder {
    seen: Mutex<Vec<Request>>,
    reply: Mutex<Value>,
    status: Mutex<u16>,
}

impl Recorder {
    fn replying(body: Value) -> Self {
        Self {
            reply: Mutex::new(body),
            status: Mutex::new(200),
            ..Self::default()
        }
    }

    fn last(&self) -> Request {
        self.seen.lock().unwrap().last().cloned().unwrap()
    }
}

#[async_trait]
impl Gateway for Recorder {
    async fn send(&self, request: Request) -> Result<Response> {
        self.seen.lock().unwrap().push(request.clone());
        let status = *self.status.lock().unwrap();
        if status >= 400 {
            return Err(Error::Api {
                status,
                method: request.method,
                path: request.path.as_str().to_owned(),
                message: "denied".to_owned(),
                detail: None,
            });
        }
        Ok(Response {
            status,
            body: self.reply.lock().unwrap().clone(),
            text: None,
            location: None,
        })
    }

    fn base_url(&self) -> &'static str {
        "https://kc.test"
    }
}

#[tokio::test]
async fn saving_preserves_fields_the_tool_does_not_model() {
    let gateway = Recorder::replying(Value::Null);
    let users = resources::by_id("users").unwrap();

    let from_server = json!({
        "id": "u1",
        "username": "alice",
        "enabled": true,
        "aFieldFromTheFuture": {"deeply": ["nested"]}
    });

    gateway
        .save(
            users,
            &Context::realm("acme"),
            "u1",
            &from_server,
            json!({"enabled": false}),
        )
        .await
        .unwrap();

    let sent = gateway.last().body.unwrap();
    assert_eq!(sent["enabled"], json!(false), "the edit must be applied");
    assert_eq!(
        sent["aFieldFromTheFuture"]["deeply"],
        json!(["nested"]),
        "unknown fields must survive the round-trip"
    );
    assert_eq!(gateway.last().path.as_str(), "/admin/realms/acme/users/u1");
}

#[tokio::test]
async fn listing_a_nested_component_collection_sends_the_parent_filter() {
    let gateway = Recorder::replying(json!([]));
    let mappers = resources::by_id("federation-mappers").unwrap();

    let context = Context::realm("acme").with("component", "ldap-1");
    gateway
        .list(mappers, &context, &Query::new())
        .await
        .unwrap();

    let sent = gateway.last();
    assert_eq!(sent.path.as_str(), "/admin/realms/acme/components");
    assert!(
        sent.query
            .contains(&("parent".to_owned(), "ldap-1".to_owned()))
    );
}

#[tokio::test]
async fn responses_wrapped_in_an_object_still_produce_rows() {
    let gateway = Recorder::replying(json!({
        "keys": [{"kid": "a", "algorithm": "RS256"}, {"kid": "b", "algorithm": "RS512"}],
        "active": {"RS256": "a"}
    }));
    let keys = resources::by_id("realm-keys").unwrap();

    let page = gateway
        .list(keys, &Context::realm("acme"), &Query::new())
        .await
        .unwrap();

    assert_eq!(page.len(), 2);
    assert_eq!(page.items[0]["kid"], json!("a"));
}

#[tokio::test]
async fn a_403_on_serverinfo_degrades_instead_of_failing_the_session() {
    // Exactly what a realm-scoped admin hits on Keycloak 26.7.
    #[derive(Debug)]
    struct ForbiddenServerInfo;

    #[async_trait]
    impl Gateway for ForbiddenServerInfo {
        async fn send(&self, request: Request) -> Result<Response> {
            if request.path.as_str().contains("serverinfo") {
                return Err(Error::Api {
                    status: 403,
                    method: request.method,
                    path: request.path.as_str().to_owned(),
                    message: "Forbidden".to_owned(),
                    detail: None,
                });
            }
            Ok(Response {
                status: 200,
                body: json!({"issuer": "https://kc.test/realms/acme"}),
                text: None,
                location: None,
            })
        }

        fn base_url(&self) -> &'static str {
            "https://kc.test"
        }
    }

    let profile = ForbiddenServerInfo.server_profile("acme").await.unwrap();
    assert_eq!(profile.source, crate::capability::ProfileSource::Discovery);
    assert!(profile.version.is_unknown());
    // And, critically, the operator can still use every screen.
    assert!(
        profile
            .evaluate(&crate::capability::Requirement::Feature("organization"))
            .is_permitted()
    );
}

#[test]
fn created_ids_come_from_the_location_header() {
    let response = Response {
        status: 201,
        body: Value::Null,
        text: None,
        location: Some("https://kc.test/admin/realms/r/users/abc-123".to_owned()),
    };
    assert_eq!(response.created_id().as_deref(), Some("abc-123"));
}
