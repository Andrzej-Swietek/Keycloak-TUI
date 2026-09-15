use kct_core::Secret;

use super::*;

fn credentials() -> Credentials {
    Credentials::ClientCredentials {
        client_id: "admin-cli".into(),
        client_secret: Secret::new("s"),
    }
}

#[test]
fn urls_are_normalised_and_obvious_mistakes_rejected() {
    let connection = Connection::new(
        "https://kc.example.com/",
        "master",
        "avsystem",
        credentials(),
    )
    .unwrap();
    assert_eq!(connection.base_url, "https://kc.example.com");

    assert!(Connection::new("kc.example.com", "master", "r", credentials()).is_err());
    assert!(Connection::new("", "master", "r", credentials()).is_err());
}

#[test]
fn empty_realms_fall_back_to_master() {
    let connection = Connection::new("https://kc", "", "", credentials()).unwrap();
    assert_eq!(connection.auth_realm, "master");
    assert_eq!(connection.realm, "master");
}

#[test]
fn cleartext_is_flagged_except_on_loopback() {
    let remote = Connection::new("http://kc.internal", "master", "r", credentials()).unwrap();
    assert!(remote.cleartext_warning().is_some());

    let local = Connection::new("http://localhost:8080", "master", "r", credentials()).unwrap();
    assert!(local.cleartext_warning().is_none());

    let tls = Connection::new("https://kc.internal", "master", "r", credentials()).unwrap();
    assert!(tls.cleartext_warning().is_none());
}
