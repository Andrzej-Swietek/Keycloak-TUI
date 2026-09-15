use super::*;
use kct_core::Version;
use kct_core::capability::ServerProfile;

fn modern() -> ServerProfile {
    ServerProfile::from_server_info(serde_json::json!({
        "systemInfo": {"version": "26.6.1"},
        "features": [{"name": "organization", "type": "DEFAULT", "enabled": true}]
    }))
}

fn titles(entries: &[Entry], needle: &str) -> Vec<String> {
    search(entries, needle)
        .into_iter()
        .map(|index| entries[index].title.clone())
        .collect()
}

#[test]
fn typing_an_abbreviation_finds_the_screen() {
    let entries = catalog(&modern(), &[], None);
    let found = titles(&entries, "usrfed");
    assert_eq!(found.first().unwrap(), "Go to User federation");
}

#[test]
fn domain_vocabulary_finds_things_named_differently_in_keycloak() {
    let entries = catalog(&modern(), &[], None);

    // An operator thinks "LDAP", Keycloak calls it "User federation".
    assert!(
        titles(&entries, "ldap")
            .iter()
            .any(|t| t.contains("User federation"))
    );
    // ...and "tenant" for organizations.
    assert!(
        titles(&entries, "tenant")
            .iter()
            .any(|t| t.contains("Organization"))
    );
    // ...and "2fa" for authentication flows.
    assert!(
        titles(&entries, "2fa")
            .iter()
            .any(|t| t.contains("Authentication flows"))
    );
}

#[test]
fn the_current_views_actions_rank_above_the_same_action_elsewhere() {
    let users = resources::by_id("users").unwrap();
    let entries = catalog(&modern(), &[], Some(users));

    let ranked = titles(&entries, "delete user");
    assert_eq!(ranked.first().unwrap(), "Delete user");
}

#[test]
fn unavailable_entries_stay_visible_and_say_why() {
    let old = ServerProfile::from_discovery(Version::new(25, 0, 6));
    let entries = catalog(&old, &[], None);

    let organizations = entries
        .iter()
        .find(|entry| entry.command == Command::Open("organizations"))
        .expect("still listed on an old server");

    assert!(!organizations.available);
    assert!(organizations.note.as_ref().unwrap().contains("26.0.0"));
}

#[test]
fn realms_are_searchable_by_name() {
    let realms = vec!["master".to_owned(), "avsystem".to_owned()];
    let entries = catalog(&modern(), &realms, None);

    let found = titles(&entries, "avsys");
    assert_eq!(found.first().unwrap(), "Realm: avsystem");
}

#[test]
fn an_empty_query_keeps_a_stable_useful_order() {
    let users = resources::by_id("users").unwrap();
    let entries = catalog(&modern(), &[], Some(users));
    let order = search(&entries, "");

    assert_eq!(order.len(), entries.len());
    assert_eq!(entries[order[0]].title, "Create user");
}
