use async_trait::async_trait;

use crate::error::Result;
use crate::secret::Secret;

/// Read access to the session's own credentials.
///
/// A second, deliberately tiny port: the admin API never needs this, but the token tools do —
/// inspecting what you are actually presenting to the server is half of diagnosing "why does
/// this call fail for me but not for you".
#[async_trait]
pub trait TokenSource: Send + Sync + 'static {
    /// The current access token, renewing it if necessary.
    async fn access_token(&self) -> Result<Secret>;

    /// How the session authenticated, for display.
    fn describe(&self) -> String;
}
