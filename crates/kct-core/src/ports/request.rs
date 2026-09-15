use serde_json::Value;

use crate::error::Method;
use crate::path::ApiPath;

/// What a response body is expected to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expect {
    /// Parse as JSON; a non-JSON body is an error.
    Json,
    /// Keep the body as text (adapter installation configs, SAML descriptors).
    Text,
    /// Discard the body (deletes, logouts).
    Nothing,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: ApiPath,
    pub query: Vec<(String, String)>,
    pub body: Option<Value>,
    pub expect: Expect,
    /// Form-encoded body, used only by the OIDC token endpoints.
    pub form: Option<Vec<(String, String)>>,
    /// Whether to attach the session's bearer token.
    ///
    /// Off for the OIDC protocol endpoints: sending an unrelated bearer to `/token` invites
    /// Keycloak to treat it as client authentication and fail in a confusing way.
    pub authenticated: bool,
}

impl Request {
    #[must_use]
    pub fn new(method: Method, path: ApiPath) -> Self {
        Self {
            method,
            path,
            query: Vec::new(),
            body: None,
            expect: Expect::Json,
            form: None,
            authenticated: true,
        }
    }

    /// A request that must not carry the session's bearer token.
    #[must_use]
    pub fn anonymous(self) -> Self {
        Self {
            authenticated: false,
            ..self
        }
    }

    #[must_use]
    pub fn get(path: ApiPath) -> Self {
        Self::new(Method::Get, path)
    }

    #[must_use]
    pub fn with_query(self, query: Vec<(String, String)>) -> Self {
        Self { query, ..self }
    }

    #[must_use]
    pub fn with_body(self, body: Value) -> Self {
        Self {
            body: Some(body),
            ..self
        }
    }

    #[must_use]
    pub fn with_form(self, form: Vec<(String, String)>) -> Self {
        Self {
            form: Some(form),
            ..self
        }
    }

    #[must_use]
    pub fn expecting(self, expect: Expect) -> Self {
        Self { expect, ..self }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: Value,
    /// Raw body, when [`Expect::Text`] was requested.
    pub text: Option<String>,
    /// `Location` header — how Keycloak reports the id of a freshly created object.
    pub location: Option<String>,
}

impl Response {
    #[must_use]
    pub fn empty(status: u16) -> Self {
        Self {
            status,
            body: Value::Null,
            text: None,
            location: None,
        }
    }

    /// The identifier of a created object.
    ///
    /// Keycloak returns `201` with a `Location` header and no body; the id is its last path
    /// segment. A few endpoints answer with a body instead, so both are tried.
    #[must_use]
    pub fn created_id(&self) -> Option<String> {
        self.location
            .as_deref()
            .and_then(|location| location.rsplit('/').next())
            .filter(|id| !id.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| {
                self.body
                    .get("id")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
    }
}
