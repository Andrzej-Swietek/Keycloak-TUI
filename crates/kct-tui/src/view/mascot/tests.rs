use super::*;
use crate::model::{Confirm, List, Prefs, Session};
use crate::msg::Notice;
use std::sync::Arc;

fn model() -> Model {
    Model::new(
        Session::new("https://kc", "avsystem", "admin"),
        kct_core::resources::by_id("users").unwrap(),
    )
}

#[test]
fn a_destructive_confirmation_outranks_a_leftover_smile() {
    let after_a_save = model().noticed(Notice::success("saved"));
    assert_eq!(Mood::of(&after_a_save), Mood::Happy);

    let about_to_delete = after_a_save.with_overlay(crate::model::Overlay::Confirm(Confirm {
        title: Arc::from("Delete realm"),
        body: Arc::from("cannot be undone"),
        danger: kct_core::catalog::Danger::Destructive,
        expect: Some(Arc::from("production")),
        typed: crate::TextInput::default(),
        on_confirm: Arc::new(Vec::new()),
    }));

    assert_eq!(Mood::of(&about_to_delete), Mood::Alarmed);
}

#[test]
fn a_failure_outranks_being_busy() {
    let failing = Model {
        list: List {
            loading: true,
            error: Some(kct_core::Error::other("nope")),
            ..List::default()
        },
        ..model()
    };
    assert_eq!(Mood::of(&failing), Mood::Worried);
}

#[test]
fn dry_run_is_visible_even_when_nothing_is_happening() {
    let watching = Model {
        session: Session {
            dry_run: true,
            ..Session::new("https://kc", "avsystem", "admin")
        },
        ..model()
    };
    assert_eq!(Mood::of(&watching), Mood::Sly);
}

#[test]
fn it_yields_the_corner_on_a_terminal_that_cannot_spare_it() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    let model = model();
    let mut small = Terminal::new(TestBackend::new(80, 24)).unwrap();
    small
        .draw(|frame| crate::view::render(&model, frame))
        .unwrap();

    let drawn: String = small
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    assert!(
        !drawn.contains('✚'),
        "the listing needs the room on a small terminal"
    );
}

#[test]
fn turning_it_off_hides_it_on_any_size() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    let hidden = Model {
        prefs: Prefs::hidden(),
        ..model()
    };
    let mut wide = Terminal::new(TestBackend::new(140, 40)).unwrap();
    wide.draw(|frame| crate::view::render(&hidden, frame))
        .unwrap();

    let drawn: String = wide
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    assert!(!drawn.contains('▀'));
}

#[test]
fn cycling_visits_every_character_then_hides() {
    use crate::view::Character;

    use crate::model::Mascot;

    let seen: Vec<Option<Mascot>> =
        std::iter::successors(Some(Prefs::default()), |prefs| Some(prefs.next_mascot(0)))
            .take(Character::ALL.len() + 1)
            .map(|prefs| prefs.mascot)
            .collect();

    for character in Character::ALL {
        assert!(
            seen.contains(&Some(Mascot::Builtin(character))),
            "never showed {}",
            character.name()
        );
    }
    assert!(seen.contains(&None), "cycling must reach 'hidden'");
}
