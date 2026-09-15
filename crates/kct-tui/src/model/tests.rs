use kct_core::catalog::Danger;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::text_input::TextInput;

use super::*;

fn users() -> &'static Resource {
    kct_core::resources::by_id("users").unwrap()
}

fn list_of(rows: Vec<Value>) -> List {
    List {
        rows: Arc::from(rows),
        ..List::default()
    }
    .refiltered(users())
}

#[test]
fn the_local_filter_matches_what_is_rendered_not_the_raw_json() {
    let list = list_of(vec![
        json!({"username": "alice", "enabled": true}),
        json!({"username": "bob", "enabled": false}),
    ]);

    // "DISABLED" appears nowhere in the JSON; it is what the Enabled column renders.
    let filtered = List {
        filter: TextInput::new("disabled"),
        ..list
    }
    .refiltered(users());

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered.selected().unwrap()["username"], json!("bob"));
}

#[test]
fn filtering_never_leaves_the_cursor_past_the_end() {
    let list = list_of(vec![
        json!({"username": "a"}),
        json!({"username": "b"}),
        json!({"username": "c"}),
    ])
    .moved_to(2);

    let filtered = List {
        filter: TextInput::new("a"),
        ..list
    }
    .refiltered(users());

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered.cursor, 0);
    assert!(filtered.selected().is_some());
}

#[test]
fn the_cursor_stays_inside_the_window_when_scrolling() {
    let list = list_of(
        (0..50)
            .map(|i| json!({"username": i.to_string()}))
            .collect(),
    );

    let scrolled = list.clone().moved_to(30).scrolled_into_view(10);
    assert_eq!(scrolled.scroll, 21);

    let back = scrolled.moved_to(2).scrolled_into_view(10);
    assert_eq!(back.scroll, 2);
}

#[test]
fn navigation_can_always_return_but_never_past_the_root() {
    let nav = Nav::rooted(users(), Context::realm("master"));
    assert!(nav.clone().popped().is_none());

    let deeper = nav.pushed(Location {
        resource: kct_core::resources::by_id("user-sessions").unwrap(),
        context: Context::realm("master").with("user", "u1"),
        crumb: Arc::from("alice"),
    });
    assert_eq!(deeper.crumbs(), vec!["Users", "alice"]);
    assert_eq!(deeper.popped().unwrap().depth(), 1);
}

#[test]
fn a_destructive_confirmation_requires_the_exact_name() {
    let confirm = Confirm {
        title: Arc::from("Delete realm"),
        body: Arc::from("This cannot be undone."),
        danger: Danger::Destructive,
        expect: Some(Arc::from("production")),
        typed: TextInput::new("producti"),
        on_confirm: Arc::new(Vec::new()),
    };
    assert!(!confirm.is_satisfied());

    let typed_fully = Confirm {
        typed: TextInput::new("production"),
        ..confirm
    };
    assert!(typed_fully.is_satisfied());
}

#[test]
fn failures_persist_while_routine_notices_expire() {
    let session = Session::new("https://kc", "master", "admin");
    let model = Model::new(session, users())
        .noticed(Notice::success("saved"))
        .noticed(Notice::failure("did not save"));

    // Simulate the passage of time by ageing both toasts.
    let aged = Model {
        toasts: model
            .toasts
            .into_iter()
            .map(|toast| Toast {
                at: Instant::now()
                    .checked_sub(NOTICE_TTL + std::time::Duration::from_secs(1))
                    .expect("the process has been running long enough"),
                ..toast
            })
            .collect(),
        ..model
    }
    .expire_toasts();

    assert_eq!(aged.toasts.len(), 1);
    assert_eq!(aged.toasts[0].notice.level, Level::Failure);
}

#[test]
fn standing_warnings_cover_every_way_a_session_can_be_unsafe() {
    let session = Session {
        dry_run: true,
        insecure: true,
        cleartext: Some(Arc::from("http")),
        ..Session::new("http://kc", "master", "admin")
    };
    let warnings = session.standing_warnings();
    assert!(warnings.contains(&"DRY-RUN".to_owned()));
    assert!(warnings.contains(&"TLS UNVERIFIED".to_owned()));
    assert!(warnings.contains(&"CLEARTEXT".to_owned()));
}

#[test]
fn a_detail_reports_only_what_actually_changed() {
    let detail = Detail {
        original: json!({"username": "alice", "enabled": true, "extra": 1}),
        draft: json!({"username": "alice", "enabled": false, "extra": 1}),
        ..Detail::loading("u1", "alice", 1)
    };

    assert!(detail.is_dirty());
    assert_eq!(detail.pending_changes(), json!({"enabled": false}));
}
