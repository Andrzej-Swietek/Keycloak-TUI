use async_trait::async_trait;
use serde_json::Value;

use crate::error::{Method, Result};
use crate::path::{ApiPath, PathParams, Template};

use super::gateway::Gateway;
use super::request::{Expect, Request, Response};

/// Thin HTTP verbs over the admin API, free for any [`Gateway`].
#[async_trait]
pub trait Rest: Gateway {
    async fn get_json(&self, path: ApiPath, query: Vec<(String, String)>) -> Result<Value> {
        self.send(Request::get(path).with_query(query))
            .await
            .map(|response| response.body)
    }

    async fn get_text(&self, path: ApiPath) -> Result<String> {
        self.send(Request::get(path).expecting(Expect::Text))
            .await
            .map(|response| response.text.unwrap_or_default())
    }

    async fn post_json(&self, path: ApiPath, body: Value) -> Result<Response> {
        self.send(Request::new(Method::Post, path).with_body(body))
            .await
    }

    async fn put_json(&self, path: ApiPath, body: Value) -> Result<Response> {
        self.send(
            Request::new(Method::Put, path)
                .with_body(body)
                .expecting(Expect::Nothing),
        )
        .await
    }

    async fn delete(&self, path: ApiPath) -> Result<Response> {
        self.send(Request::new(Method::Delete, path).expecting(Expect::Nothing))
            .await
    }

    async fn resolve(&self, template: Template, params: &PathParams) -> Result<ApiPath> {
        template.resolve(params)
    }
}

impl<T: Gateway + ?Sized> Rest for T {}
