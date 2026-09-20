//! I/O described as data. The runtime is the only thing that interprets these.

use std::sync::Arc;

use kct_core::catalog::Shape;
use kct_core::{ApiPath, Method};
use serde_json::Value;

use crate::msg::Fetch;

#[derive(Debug, Clone)]
pub enum Effect {
    /// Read something into the list or the detail view.
    Load {
        token: u64,
        fetch: Fetch,
        path: ApiPath,
        query: Vec<(String, String)>,
        shape: Shape,
    },
    /// Perform a write (or a read whose outcome is just "it worked").
    Call {
        method: Method,
        path: ApiPath,
        query: Vec<(String, String)>,
        body: Option<Value>,
        title: String,
    },
    /// Run a request and show its body verbatim. Carries the method: a realm export is a POST.
    Fetch {
        method: Method,
        path: ApiPath,
        query: Vec<(String, String)>,
        body: Option<Value>,
        title: String,
    },
    /// Decode the session's own access token and show its claims.
    InspectToken,
    /// Put the session's access token on the clipboard.
    CopyToken,
    /// Reproduce a request outside the tool.
    CopyCurl {
        method: Method,
        path: String,
        base_url: String,
    },
    /// Obtain a token as another client, to see what that client would actually get.
    ClientToken {
        realm: String,
        client_id: String,
        secret_path: ApiPath,
    },
    WriteFile {
        path: std::path::PathBuf,
        contents: Arc<str>,
    },
    /// Read a local file and send it as a partial import.
    ImportFile {
        source: std::path::PathBuf,
        target: ApiPath,
        policy: &'static str,
    },
    DetectCapabilities {
        realm: String,
    },
    DetectUserProfile {
        realm: String,
    },
    DetectRealmFlags {
        realm: String,
    },
    DetectAuthFlows {
        realm: String,
    },
    /// Rescan the mascot directory so new art appears without a restart.
    LoadMascots,
    ListRealms,
    /// Put text on the clipboard; `label` is what the confirmation toast says.
    Copy {
        text: String,
        label: String,
    },
    Quit,
}

impl Effect {
    /// A write with no query string, which is every write but the federation syncs.
    #[must_use]
    pub fn call(
        method: Method,
        path: ApiPath,
        body: Option<Value>,
        title: impl Into<String>,
    ) -> Self {
        Self::Call {
            method,
            path,
            query: Vec::new(),
            body,
            title: title.into(),
        }
    }
}
