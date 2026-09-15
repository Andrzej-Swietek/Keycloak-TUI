use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use kct_core::catalog::{Field, Tab};

use crate::effect::Effect;
use crate::model::{Detail, Model, Overlay};

use super::field_edit::{begin_edit, edit};
use super::nested;
use super::{nav, transition::Transition};

/// Fields of a form tab the current server actually supports.
#[must_use]
pub fn visible_fields(model: &Model, tab: &Tab) -> Vec<&'static Field> {
    match tab {
        Tab::Form { fields, .. } => fields
            .iter()
            .filter(|field| {
                model
                    .session
                    .profile
                    .evaluate(&field.requirement)
                    .is_permitted()
            })
            .collect(),
        _ => Vec::new(),
    }
}

#[must_use]
pub fn handle(model: Model, key: KeyEvent) -> Transition {
    let Some(detail) = model.detail.clone() else {
        return Transition::pure(model);
    };

    if detail.editing.is_some() {
        return edit(model, detail, key);
    }

    let tabs = model.resource().tabs;
    let control = key.modifiers.contains(KeyModifiers::CONTROL);

    match (key.code, control) {
        (KeyCode::Esc, _) => nav::back(model),
        (KeyCode::Char('h') | KeyCode::Left, false) => {
            Transition::pure(tab_moved(model, detail, -1))
        }
        (KeyCode::Char('l') | KeyCode::Right, false) => {
            Transition::pure(tab_moved(model, detail, 1))
        }
        (KeyCode::Char('j') | KeyCode::Down, false) => {
            Transition::pure(cursor_moved(model, detail, 1))
        }
        (KeyCode::Char('k') | KeyCode::Up, false) => {
            Transition::pure(cursor_moved(model, detail, -1))
        }
        (KeyCode::Enter, _) => match tabs.get(detail.tab) {
            Some(tab @ Tab::Nested { .. }) => nested::enter_nested(model, tab),
            Some(Tab::Form { .. } | Tab::Raw) => begin_edit(model, detail),
            None => Transition::pure(model),
        },
        // The command line, help and log are global; a detail view must not swallow them.
        (KeyCode::Char(':'), false) | (KeyCode::Char('p'), true) => {
            super::commands::open_palette(model)
        }
        (KeyCode::Char('?'), false) => {
            Transition::pure(model.with_overlay(Overlay::Help { scroll: 0 }))
        }
        (KeyCode::Char('L'), false) => {
            Transition::pure(model.with_overlay(Overlay::RequestLog { cursor: 0 }))
        }
        (KeyCode::Char('a'), false) => super::commands::open_palette(model),
        (KeyCode::Char('w'), false) => super::save::save(model, detail),
        (KeyCode::Char('r'), false) => nav::open_selected(Model {
            detail: None,
            ..model
        }),
        (KeyCode::Char('y'), false) => Transition::with(
            model,
            Effect::Copy {
                text: kct_core::json::pretty(&detail.draft),
                label: "object JSON".to_owned(),
            },
        ),
        _ => Transition::pure(model),
    }
}

fn tab_moved(model: Model, detail: Detail, delta: i32) -> Model {
    let count = model.resource().tabs.len();
    if count == 0 {
        return model;
    }
    let count = i64::try_from(count).unwrap_or(1);
    let current = i64::try_from(detail.tab).unwrap_or(0);
    let tab = usize::try_from((current + i64::from(delta)).rem_euclid(count)).unwrap_or(0);

    Model {
        detail: Some(Detail {
            tab,
            cursor: 0,
            scroll: 0,
            ..detail
        }),
        ..model
    }
}

fn cursor_moved(model: Model, detail: Detail, delta: isize) -> Model {
    let last = match model.resource().tabs.get(detail.tab) {
        Some(Tab::Raw) => super::raw::leaves(&detail).len(),
        Some(tab) => visible_fields(&model, tab).len(),
        None => 0,
    }
    .saturating_sub(1);

    let cursor = detail.cursor.saturating_add_signed(delta).min(last);
    Model {
        detail: Some(Detail { cursor, ..detail }),
        ..model
    }
}
