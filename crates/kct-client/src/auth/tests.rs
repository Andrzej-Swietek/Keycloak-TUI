use super::*;
use std::sync::Mutex as StdMutex;

use serde_json::json;

#[derive(Debug, Default)]
struct FakeEndpoint {
    calls: StdMutex<Vec<Vec<(String, String)>>>,
    replies: StdMutex<Vec<Result<Value>>>,
}

impl FakeEndpoint {
    fn with(replies: Vec<Result<Value>>) -> Arc<Self> {
        Arc::new(Self {
            calls: StdMutex::new(Vec::new()),
            replies: StdMutex::new(replies),
        })
    }

    fn grant_types(&self) -> Vec<String> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter_map(|form| {
                form.iter()
                    .find(|(k, _)| k == "grant_type")
                    .map(|(_, v)| v.clone())
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl TokenEndpoint for FakeEndpoint {
    async fn post_form(&self, _path: String, form: Vec<(String, String)>) -> Result<Value> {
        self.calls.lock().unwrap().push(form);
        let mut replies = self.replies.lock().unwrap();
        if replies.is_empty() {
            Ok(json!({"access_token": "default", "expires_in": 300}))
        } else {
            replies.remove(0)
        }
    }
}

fn password_credentials() -> Credentials {
    Credentials::Password {
        client_id: "admin-cli".into(),
        username: "admin".into(),
        password: Secret::new("admin"),
        client_secret: None,
        otp: None,
    }
}

#[tokio::test]
async fn the_first_call_authenticates_and_later_calls_reuse_the_token() {
    let endpoint = FakeEndpoint::with(vec![Ok(
        json!({"access_token": "tok-1", "expires_in": 300, "refresh_token": "ref-1", "refresh_expires_in": 1800}),
    )]);
    let auth = Authenticator::new(endpoint.clone(), "master", password_credentials());

    assert_eq!(auth.bearer().await.unwrap().expose(), "tok-1");
    assert_eq!(auth.bearer().await.unwrap().expose(), "tok-1");
    assert_eq!(endpoint.grant_types(), vec!["password"]);
}

#[tokio::test]
async fn an_expired_token_is_renewed_with_the_refresh_grant_not_the_password() {
    let endpoint = FakeEndpoint::with(vec![
        Ok(
            json!({"access_token": "tok-1", "expires_in": 300, "refresh_token": "ref-1", "refresh_expires_in": 1800}),
        ),
        Ok(
            json!({"access_token": "tok-2", "expires_in": 300, "refresh_token": "ref-2", "refresh_expires_in": 1800}),
        ),
    ]);
    let auth = Authenticator::new(endpoint.clone(), "master", password_credentials());

    assert_eq!(auth.bearer().await.unwrap().expose(), "tok-1");
    auth.invalidate().await;
    assert_eq!(auth.bearer().await.unwrap().expose(), "tok-2");

    assert_eq!(
        endpoint.grant_types(),
        vec!["password", "refresh_token"],
        "re-sending the password on every renewal would trip brute-force detection"
    );
}

#[tokio::test]
async fn a_revoked_refresh_token_falls_back_to_full_authentication() {
    let endpoint = FakeEndpoint::with(vec![
        Ok(
            json!({"access_token": "tok-1", "expires_in": 300, "refresh_token": "ref-1", "refresh_expires_in": 1800}),
        ),
        Err(Error::Auth(AuthError::OAuth {
            error: "invalid_grant".to_owned(),
            description: Some("Session not active".to_owned()),
        })),
        Ok(json!({"access_token": "tok-3", "expires_in": 300})),
    ]);
    let auth = Authenticator::new(endpoint.clone(), "master", password_credentials());

    auth.bearer().await.unwrap();
    auth.invalidate().await;

    assert_eq!(auth.bearer().await.unwrap().expose(), "tok-3");
    assert_eq!(
        endpoint.grant_types(),
        vec!["password", "refresh_token", "password"]
    );
}

#[tokio::test]
async fn a_pre_issued_token_is_used_as_is_and_never_renewed() {
    let endpoint = FakeEndpoint::with(vec![]);
    let auth = Authenticator::new(
        endpoint.clone(),
        "master",
        Credentials::Bearer(Secret::new("from-ci")),
    );

    assert_eq!(auth.bearer().await.unwrap().expose(), "from-ci");
    assert_eq!(endpoint.grant_types(), Vec::<String>::new());

    auth.invalidate().await;
    assert!(auth.bearer().await.is_err(), "nothing to renew from");
}

#[tokio::test]
async fn concurrent_requests_trigger_exactly_one_token_call() {
    let endpoint = FakeEndpoint::with(vec![]);
    let auth = Arc::new(Authenticator::new(
        endpoint.clone(),
        "master",
        password_credentials(),
    ));

    let calls = (0..16).map(|_| {
        let auth = auth.clone();
        tokio::spawn(async move { auth.bearer().await.map(|t| t.expose().to_owned()) })
    });
    let results = futures_join(calls).await;

    assert!(results.iter().all(|token| token == "default"));
    assert_eq!(
        endpoint.grant_types().len(),
        1,
        "a token stampede looks like an attack to a brute-force-protected realm"
    );
}

async fn futures_join(
    handles: impl Iterator<Item = tokio::task::JoinHandle<Result<String>>>,
) -> Vec<String> {
    let mut out = Vec::new();
    for handle in handles.collect::<Vec<_>>() {
        out.push(handle.await.unwrap().unwrap());
    }
    out
}

#[test]
fn bad_passwords_become_a_typed_error_not_a_raw_400() {
    let error = oauth_error(
        &json!({"error": "invalid_grant", "error_description": "Invalid user credentials"}),
        "admin-cli",
        "master",
    );
    assert!(matches!(
        error,
        Error::Auth(AuthError::InvalidCredentials { .. })
    ));
    assert!(error.hint().is_some());
}

#[test]
fn a_token_response_without_a_token_is_rejected() {
    let session = Session::from_token_response(&json!({"expires_in": 300}));
    assert!(session.is_err());
}
