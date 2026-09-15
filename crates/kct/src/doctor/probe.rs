use std::collections::BTreeMap;

use kct_core::catalog::Resource;
use kct_core::ports::{AdminApi, Context};
use kct_core::{Query, resources};
use serde_json::Value;

/// One real object of each kind, supplying the parent ids nested collections are probed with.
#[derive(Debug, Default)]
pub struct Parents {
    ids: BTreeMap<&'static str, String>,
    /// Authorization endpoints exist only on a client that is a resource server; probing any
    /// other client answers 404 and says nothing about the catalog.
    authz_client: Option<String>,
}

impl Parents {
    /// Sample the realm for a client, scope, group, identity provider and component.
    pub async fn sample(api: &impl AdminApi, realm: &Context) -> Self {
        let mut ids = BTreeMap::new();
        for (slot, resource_id, key) in [
            ("client", "clients", "id"),
            ("scope", "client-scopes", "id"),
            ("group", "groups", "id"),
            ("alias", "identity-providers", "alias"),
            ("component", "user-federation", "id"),
            ("org", "organizations", "id"),
            ("flow", "auth-flows", "alias"),
            ("role", "realm-roles", "name"),
            ("user", "users", "id"),
        ] {
            let Some(resource) = resources::by_id(resource_id) else {
                continue;
            };
            let sampled = api
                .list(resource, realm, &Query::page(0, 1))
                .await
                .ok()
                .and_then(|page| {
                    page.items
                        .first()
                        .and_then(|row| row.get(key))
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                });
            if let Some(id) = sampled {
                ids.insert(slot, id);
            }
        }
        let authz_client = match resources::by_id("clients") {
            Some(clients) => api
                .list(clients, realm, &Query::page(0, 100))
                .await
                .ok()
                .and_then(|page| {
                    page.items
                        .iter()
                        .find(|client| client["authorizationServicesEnabled"] == Value::Bool(true))
                        .and_then(|client| client["id"].as_str())
                        .map(ToOwned::to_owned)
                }),
            None => None,
        };

        Self { ids, authz_client }
    }

    /// A context for probing this resource, or `None` when the realm has no suitable parent.
    #[must_use]
    pub fn context_for(&self, resource: &Resource, realm: &Context) -> Option<Context> {
        let authz = resource.id.contains("authz");

        resource
            .collection_slots()
            .into_iter()
            .try_fold(realm.clone(), |context, slot| {
                let id = match (slot, authz) {
                    ("client", true) => self.authz_client.as_ref(),
                    _ => self.ids.get(slot),
                }?;
                Some(context.with(slot, id.clone()))
            })
    }

    /// Role mappings need to know whether they are serving a user or a group.
    #[must_use]
    pub fn with_owner_kind(&self, context: Context, resource: &Resource) -> Context {
        if resource.collection_slots().contains(&"owner") {
            return context.with("ownerKind", "users");
        }
        context
    }

    #[must_use]
    pub fn owner_context(&self, realm: &Context) -> Option<Context> {
        self.ids.get("user").map(|id| {
            realm
                .clone()
                .with("ownerKind", "users")
                .with("owner", id.clone())
        })
    }
}
