use async_trait::async_trait;
use serde_json::Value;

use crate::capability::ServerProfile;
use crate::catalog::Resource;
use crate::error::{Error, Result};
use crate::json;
use crate::page::Page;
use crate::path::PathParams;
use crate::plan;
use crate::query::Query;

use super::context::Context;
use super::rest::Rest;

/// Everything the application actually calls. Implemented for free by any [`super::Gateway`].
#[async_trait]
pub trait AdminApi: Rest {
    // -- catalog-driven operations ------------------------------------------------------

    /// List one page of a catalog resource.
    async fn list(
        &self,
        resource: &'static Resource,
        context: &Context,
        query: &Query,
    ) -> Result<Page<Value>> {
        let planned = plan::list(resource, context, query)?;
        let body = self.get_json(planned.path, planned.query).await?;

        Ok(Page::new(resource.shape.rows(body), query.clone()))
    }

    /// Fetch one object in full.
    ///
    /// Always a complete representation: list rows may have come from a `briefRepresentation`
    /// request and are not safe to write back.
    async fn fetch(
        &self,
        resource: &'static Resource,
        context: &Context,
        id: &str,
    ) -> Result<Value> {
        let planned = plan::item(resource, context, id)?;
        self.get_json(planned.path, planned.query).await
    }

    /// `PUT` `original` merged with `patch`.
    ///
    /// `original` is the representation as fetched and `patch` only the changed fields, so
    /// keys this build does not model are written back unchanged.
    async fn save(
        &self,
        resource: &'static Resource,
        context: &Context,
        id: &str,
        original: &Value,
        patch: Value,
    ) -> Result<()> {
        let merged = json::merge(original.clone(), patch);
        let planned = plan::item(resource, context, id)?;
        self.put_json(planned.path, merged).await.map(|_| ())
    }

    /// Create an object, returning its server-assigned id when one is reported.
    async fn create(
        &self,
        resource: &'static Resource,
        context: &Context,
        body: Value,
    ) -> Result<Option<String>> {
        if resource.embeds_items() {
            let id = resource.key_of(&body).map(ToOwned::to_owned);
            self.rewrite_document(
                resource,
                context,
                id.as_deref().unwrap_or_default(),
                Some(body),
            )
            .await?;
            return Ok(id);
        }

        let path = resource.collection.resolve(&context.params)?;
        self.post_json(path, body)
            .await
            .map(|response| response.created_id())
    }

    async fn remove(&self, resource: &'static Resource, context: &Context, id: &str) -> Result<()> {
        if resource.embeds_items() {
            return self.rewrite_document(resource, context, id, None).await;
        }

        let planned = plan::item(resource, context, id)?;
        self.delete(planned.path).await.map(|_| ())
    }

    /// Set `id` to `element` in the collection document, or remove it when `element` is `None`.
    ///
    /// Reads the document, applies [`Resource::with_item`] and writes it back. Does nothing
    /// unless [`Resource::embeds_items`].
    async fn rewrite_document(
        &self,
        resource: &'static Resource,
        context: &Context,
        id: &str,
        element: Option<Value>,
    ) -> Result<()> {
        let path = resource.collection.resolve(&context.params)?;
        let document = self.get_json(path.clone(), Vec::new()).await?;
        match resource.with_item(document, id, element) {
            Some(next) => self.put_json(path, next).await.map(|_| ()),
            None => Ok(()),
        }
    }

    // -- server capabilities -------------------------------------------------------------

    /// Detect what this server can do.
    ///
    /// Reads `serverinfo`, falling back to the realm's public discovery document for the
    /// version alone and returning a profile whose [`crate::capability::ProfileSource`] reports it as partial.
    /// Errs when neither document can be read.
    ///
    /// A realm-scoped account gets 403 on `serverinfo` from Keycloak 26.7, which dropped the
    /// `view-system` role.
    async fn server_profile(&self, realm: &str) -> Result<ServerProfile> {
        use crate::path::endpoints as ep;

        match self
            .get_json(ep::SERVER_INFO.resolve(&PathParams::none())?, Vec::new())
            .await
        {
            Ok(body) => Ok(ServerProfile::from_server_info(body)),
            Err(error) if matches!(error.status(), Some(401 | 403 | 404)) => {
                let discovery = ep::OIDC_DISCOVERY.resolve(&PathParams::realm(realm))?;
                let _ = self.get_json(discovery, Vec::new()).await?;
                // Discovery carries no version field; the profile records that it is partial.
                Ok(ServerProfile::from_discovery(
                    crate::version::Version::UNKNOWN,
                ))
            }
            Err(error) => Err(error),
        }
    }

    /// Realms this account can see, by name.
    async fn realms(&self) -> Result<Vec<String>> {
        use crate::path::endpoints as ep;

        let body = self
            .get_json(
                ep::REALMS.resolve(&PathParams::none())?,
                vec![("briefRepresentation".to_owned(), "true".to_owned())],
            )
            .await?;

        Ok(body
            .as_array()
            .map(|realms| {
                realms
                    .iter()
                    .filter_map(|realm| realm.get("realm").and_then(Value::as_str))
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Cheap liveness probe used by the connection screen.
    async fn ping(&self, realm: &str) -> Result<()> {
        use crate::path::endpoints as ep;

        self.get_json(
            ep::OIDC_DISCOVERY.resolve(&PathParams::realm(realm))?,
            Vec::new(),
        )
        .await
        .map(|_| ())
        .map_err(|error| match error {
            Error::Api { status: 404, .. } => Error::Config(format!(
                "no realm named '{realm}' at {} (or the base URL is wrong)",
                self.base_url()
            )),
            other => other,
        })
    }
}

impl<T: Rest + ?Sized> AdminApi for T {}
