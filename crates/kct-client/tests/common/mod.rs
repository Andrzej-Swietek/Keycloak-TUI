use kct_client::{Connection, Credentials, KeycloakClient};
use kct_core::Secret;

/// A client pointed at the server named by `KCT_TEST_SERVER`, or `None` when unset.
pub fn client() -> Option<KeycloakClient> {
    let server = std::env::var("KCT_TEST_SERVER").ok()?;
    let connection = Connection::new(
        server,
        "master",
        std::env::var("KCT_TEST_REALM").unwrap_or_else(|_| "master".into()),
        Credentials::Password {
            client_id: "admin-cli".into(),
            username: std::env::var("KCT_TEST_USER").unwrap_or_else(|_| "admin".into()),
            password: Secret::new(
                std::env::var("KCT_TEST_PASSWORD").unwrap_or_else(|_| "admin".into()),
            ),
            client_secret: None,
            otp: None,
        },
    )
    .expect("valid connection");

    Some(KeycloakClient::connect(connection).expect("client builds"))
}
