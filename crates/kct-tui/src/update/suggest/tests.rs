use super::*;

fn realm() -> &'static Resource {
    kct_core::resources::by_id("realm-settings").unwrap()
}
use kct_core::capability::ServerProfile;
use serde_json::json;

fn session_with_themes() -> Session {
    Session {
        profile: ServerProfile::from_server_info(json!({
            "systemInfo": {"version": "26.7.3"},
            "themes": {
                "login": [{"name": "keycloak"}, {"name": "keycloak.v2"}, {"name": "base"}]
            }
        })),
        ..Session::new("https://kc", "master", "admin")
    }
}

#[test]
fn a_choice_field_suggests_exactly_its_options() {
    let field = Field::new(
        "SSL required",
        "/sslRequired",
        FieldKind::Choice(&["all", "external", "none"]),
    );
    assert_eq!(
        for_field(&field, realm(), &session_with_themes()),
        vec!["all", "external", "none"]
    );
}

#[test]
fn theme_fields_offer_what_the_server_actually_has() {
    let field = Field::new("Login theme", "/loginTheme", FieldKind::Text);
    let themes = for_field(&field, realm(), &session_with_themes());

    assert!(themes.contains(&"keycloak.v2".to_owned()));
    assert!(!themes.contains(&"a-theme-nobody-installed".to_owned()));
}

#[test]
fn a_field_with_no_known_vocabulary_suggests_nothing() {
    let field = Field::new("Description", "/description", FieldKind::Multiline);
    assert!(for_field(&field, realm(), &session_with_themes()).is_empty());
}

#[test]
fn typing_narrows_the_list_without_reordering_it() {
    let all: Vec<String> = ["RS256", "RS512", "ES256"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    assert_eq!(matching(&all, "rs"), vec![0, 1]);
    assert_eq!(matching(&all, "256"), vec![0, 2]);
    assert_eq!(matching(&all, ""), vec![0, 1, 2]);
}

