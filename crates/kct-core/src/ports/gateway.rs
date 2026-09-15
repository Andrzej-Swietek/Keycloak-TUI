use std::sync::Arc;

use async_trait::async_trait;

use crate::error::Result;

use super::request::{Request, Response};

/// The single port an adapter implements.
#[async_trait]
pub trait Gateway: Send + Sync + 'static {
    /// Send one request. Implementations are responsible for authentication, retries on
    /// transient failures, and mapping HTTP statuses onto [`crate::Error`].
    async fn send(&self, request: Request) -> Result<Response>;

    /// Base URL, for display and for building browser-facing links.
    fn base_url(&self) -> &str;
}

#[async_trait]
impl Gateway for Arc<dyn Gateway> {
    async fn send(&self, request: Request) -> Result<Response> {
        (**self).send(request).await
    }

    fn base_url(&self) -> &str {
        (**self).base_url()
    }
}
