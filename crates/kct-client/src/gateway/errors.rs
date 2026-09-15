use kct_core::Error;
use kct_core::ports::Request;
use serde_json::Value;

/// Largest slice of a response body carried into an [`crate::Error`].
///
/// Keycloak reflects parts of a request into some errors, including a grant password.
pub(super) const MAX_ERROR_BODY: usize = 1_500;

pub(super) fn transport_error(url: &str, error: &reqwest::Error) -> Error {
    let reason = if error.is_timeout() {
        "timed out".to_owned()
    } else if error.is_connect() {
        format!("connection failed ({error})")
    } else {
        error.to_string()
    };

    Error::Transport {
        url: url.to_owned(),
        reason,
    }
}

/// Turn a failed response into an error carrying Keycloak's own explanation.
///
/// Keycloak reports errors under at least four different keys depending on the endpoint;
/// trying all of them is the difference between "409 Conflict" and "User exists with same
/// username".
pub(super) fn api_error(request: &Request, status: u16, body: &str) -> Error {
    let parsed: Option<Value> = serde_json::from_str(body).ok();

    let message = parsed
        .as_ref()
        .and_then(|value| {
            ["errorMessage", "error_description", "error", "message"]
                .iter()
                .find_map(|key| value.get(*key).and_then(Value::as_str))
        })
        .map_or_else(
            || match status {
                401 => "not authenticated".to_owned(),
                403 => "forbidden".to_owned(),
                404 => "not found".to_owned(),
                409 => "conflict".to_owned(),
                _ => format!("HTTP {status}"),
            },
            ToOwned::to_owned,
        );

    let detail =
        (!body.trim().is_empty()).then(|| body.chars().take(MAX_ERROR_BODY).collect::<String>());

    Error::Api {
        status,
        method: request.method,
        path: request.path.as_str().to_owned(),
        message,
        detail,
    }
}
