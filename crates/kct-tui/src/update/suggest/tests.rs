use super::*;

fn realm() -> &'static Resource {
    kct_core::resources::by_id("realm-settings").unwrap()
}
use kct_core::capability::ServerProfile;
use serde_json::json;

fn session_with_themes() -> Session {
    Session {
        profile: ServerProfile::from_server_info(json!({
            "systemInfo": {"version": "26.8.1"},
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

#[test]
fn a_provider_a_deployed_extension_registers_is_offered() {
    // An SPI provider the deployment adds reaches the operator without a code change: the
    // suggestion reads what the server reports, not a list compiled in here.
    let session = Session {
        profile: ServerProfile::from_server_info(json!({
            "systemInfo": {"version": "26.6.1"},
            "providers": {
                "eventsListener": {
                    "providers": {
                        "jboss-logging": {},
                        "group-permission-cascade": {},
                    }
                },
                "policy": {"providers": {"role": {}, "group-hierarchy": {}}},
            },
            "protocolMapperTypes": {
                "openid-connect": [{"id": "oidc-usermodel-attribute-mapper"}, {"id": "acme-custom-mapper"}]
            },
        })),
        ..Session::new("https://kc", "acme", "admin")
    };

    let listeners = for_field(
        &Field::new("Event listeners", "/eventsListeners", FieldKind::List),
        realm(),
        &session,
    );
    assert!(
        listeners.contains(&"group-permission-cascade".to_owned()),
        "{listeners:?}"
    );

    let mappers = for_field(
        &Field::new("Mapper type", "/protocolMapper", FieldKind::Text),
        realm(),
        &session,
    );
    assert!(
        mappers.contains(&"acme-custom-mapper".to_owned()),
        "{mappers:?}"
    );
}
