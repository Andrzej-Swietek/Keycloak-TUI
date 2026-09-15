use async_trait::async_trait;
use kct_core::ports::{Expect, Request, Response};
use kct_core::{ApiPath, Error, Method, Result, Secret};
use serde_json::Value;

use crate::auth::{TokenEndpoint, oauth_error};
use crate::config::Connection;

use super::errors::{api_error, transport_error};

/// Bare HTTP, with no notion of authentication. Split from [`crate::KeycloakClient`] so the
/// authenticator can use it to fetch tokens without the two holding each other alive.
#[derive(Debug)]
pub struct HttpTransport {
    http: reqwest::Client,
    base_url: String,
}

impl HttpTransport {
    pub fn new(connection: &Connection) -> Result<Self> {
        let builder = reqwest::Client::builder()
            .user_agent(concat!("kct/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(connection.timeouts.connect)
            .timeout(connection.timeouts.request)
            .gzip(true)
            // Never follow a redirect: a 3xx on an admin call would re-send the bearer
            // token — or, on the token endpoint, the password itself — to wherever the
            // redirect points.
            .redirect(reqwest::redirect::Policy::none());

        // Extra roots are *added* to the platform trust store, never replacing it: an
        // internal CA is usually needed alongside the public ones, not instead of them.
        let extra_roots = connection
            .tls
            .ca_bundle
            .as_deref()
            .map(|pem| {
                reqwest::Certificate::from_pem_bundle(pem)
                    .map_err(|e| Error::config(format!("could not read the CA bundle: {e}")))
            })
            .transpose()?
            .unwrap_or_default();

        let builder = extra_roots
            .into_iter()
            .fold(builder, reqwest::ClientBuilder::add_root_certificate);

        let builder = if connection.tls.insecure {
            tracing::warn!("TLS verification disabled for {}", connection.base_url);
            builder.danger_accept_invalid_certs(true)
        } else {
            builder
        };

        Ok(Self {
            http: builder
                .build()
                .map_err(|e| Error::config(format!("could not build the HTTP client: {e}")))?,
            base_url: connection.base_url.clone(),
        })
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub(super) async fn execute(
        &self,
        request: &Request,
        bearer: Option<&Secret>,
    ) -> Result<Response> {
        let url = format!("{}{}", self.base_url, request.path);

        let method = match request.method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Patch => reqwest::Method::PATCH,
        };

        let builder = self.http.request(method, &url).query(&request.query);
        let builder = match bearer {
            Some(token) => builder.bearer_auth(token.expose()),
            None => builder,
        };
        let builder = match (&request.form, &request.body) {
            (Some(form), _) => builder.form(form),
            (None, Some(body)) => builder.json(body),
            (None, None) => builder,
        };

        let started = std::time::Instant::now();
        let response = builder
            .send()
            .await
            .map_err(|error| transport_error(&url, &error))?;
        let status = response.status().as_u16();

        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);

        let text = response
            .text()
            .await
            .map_err(|error| transport_error(&url, &error))?;

        tracing::debug!(
            method = %request.method,
            path = %request.path,
            status,
            bytes = text.len(),
            elapsed_ms = started.elapsed().as_millis(),
            "admin api call"
        );

        if status >= 400 {
            return Err(api_error(request, status, &text));
        }
        if (300..400).contains(&status) {
            return Err(Error::Api {
                status,
                method: request.method,
                path: request.path.as_str().to_owned(),
                message: format!(
                    "unexpected redirect to {}",
                    location.as_deref().unwrap_or("an unknown location")
                ),
                detail: Some(
                    "Redirects are not followed, because doing so would leak the bearer token."
                        .to_owned(),
                ),
            });
        }

        let body = match request.expect {
            Expect::Nothing | Expect::Text => Value::Null,
            Expect::Json if text.trim().is_empty() => Value::Null,
            Expect::Json => serde_json::from_str(&text).map_err(|error| Error::Decode {
                path: request.path.as_str().to_owned(),
                reason: error.to_string(),
            })?,
        };

        Ok(Response {
            status,
            body,
            text: matches!(request.expect, Expect::Text).then_some(text),
            location,
        })
    }
}

#[async_trait]
impl TokenEndpoint for HttpTransport {
    async fn post_form(&self, path: String, form: Vec<(String, String)>) -> Result<Value> {
        let request = Request::new(Method::Post, ApiPath::new(path)).with_form(form);

        match self.execute(&request, None).await {
            Ok(response) => Ok(response.body),
            // The token endpoint reports failures as a JSON OAuth error, which carries far
            // more than the status code does.
            Err(Error::Api { detail, .. }) => {
                let body = detail
                    .as_deref()
                    .and_then(|text| serde_json::from_str::<Value>(text).ok())
                    .unwrap_or(Value::Null);
                Err(oauth_error(&body, "", ""))
            }
            Err(other) => Err(other),
        }
    }
}
