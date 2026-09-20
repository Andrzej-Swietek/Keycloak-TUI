use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use kct_core::catalog::Verb;

use crate::effect::Effect;
use crate::model::{List, Model, Overlay, sidebar_rows};
use crate::text_input::TextInput;

use super::commands::{copy_selection, open_palette, paged};
use super::{detail, nav, overlay, transition::Transition};

/// Route a keypress to whichever layer currently owns the keyboard.
#[must_use]
pub fn handle(model: Model, key: KeyEvent) -> Transition {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Transition::with(
            Model {
                quit: true,
                ..model
            },
            Effect::Quit,
        );
    }
    if model.overlay.is_some() {
        return overlay::handle(model, key);
    }
    if model.list.filtering {
        return filtering(model, key);
    }
    if model.detail.is_some() {
        return detail::handle(model, key);
    }
    listing(model, key)
}

fn filtering(model: Model, key: KeyEvent) -> Transition {
    let resource = model.resource();
    let viewport = model.viewport_rows;

    match key.code {
        KeyCode::Esc => Transition::pure(Model {
            list: List {
                filter: TextInput::default(),
                filtering: false,
                ..model.list
            }
            .refiltered(resource),
            ..model
        }),
        KeyCode::Enter => Transition::pure(Model {
            list: List {
                filtering: false,
                ..model.list
            },
            ..model
        }),
        _ => Transition::pure(Model {
            list: List {
                filter: model.list.filter.clone().apply(key),
                ..model.list
            }
            .refiltered(resource)
            .scrolled_into_view(viewport),
            ..model
        }),
    }
}

fn listing(model: Model, key: KeyEvent) -> Transition {
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    let page = isize::try_from(model.viewport_rows.max(1)).unwrap_or(isize::MAX / 4);

    match (key.code, control) {
        (KeyCode::Char('q'), false) => Transition::with(
            Model {
                quit: true,
                ..model
            },
            Effect::Quit,
        ),
        (KeyCode::Char(':' | 'p'), _) if control || key.code == KeyCode::Char(':') => {
            open_palette(model)
        }
        (KeyCode::Char('?'), false) => {
            Transition::pure(model.with_overlay(Overlay::Help { scroll: 0 }))
        }
        (KeyCode::Char('L'), false) => {
            Transition::pure(model.with_overlay(Overlay::RequestLog { cursor: 0 }))
        }

        (KeyCode::Tab, _) => {
            let focused = !model.nav.sidebar_focused;
            Transition::pure(Model {
                nav: model.nav.clone().with_sidebar_focus(focused),
                ..model
            })
        }
        (KeyCode::Esc, _) => nav::back(model),
        (KeyCode::Char('/'), false) => Transition::pure(Model {
            list: List {
                filtering: true,
                ..model.list
            },
            ..model
        }),

        (KeyCode::Char('j') | KeyCode::Down, false) => moved(model, 1),
        (KeyCode::Char('k') | KeyCode::Up, false) => moved(model, -1),
        (KeyCode::Char('d'), true) => moved(model, page / 2),
        (KeyCode::Char('u'), true) => moved(model, -page / 2),
        (KeyCode::PageDown, _) => moved(model, page),
        (KeyCode::PageUp, _) => moved(model, -page),
        (KeyCode::Char('g') | KeyCode::Home, false) => moved(model, isize::MIN / 2),
        (KeyCode::Char('G') | KeyCode::End, false) => moved(model, isize::MAX / 2),

        (KeyCode::Enter | KeyCode::Right | KeyCode::Char('l'), false) => enter(model),
        (KeyCode::Char('a'), false) => open_palette(model),
        (KeyCode::Char('r'), false) => nav::reload(model),
        (KeyCode::Char('n'), false) => super::verbs::verb(model, Verb::Create),
        (KeyCode::Char('x'), false) => super::verbs::verb(model, Verb::Delete),
        (KeyCode::Char('y'), false) => copy_selection(model),
        (KeyCode::Char(']'), false) => paged(model, true),
        (KeyCode::Char('['), false) => paged(model, false),
        _ => Transition::pure(model),
    }
}

fn moved(model: Model, delta: isize) -> Transition {
    if model.nav.sidebar_focused {
        let rows = sidebar_rows();
        let cursor = model
            .nav
            .sidebar_cursor
            .saturating_add_signed(delta)
            .min(rows.len().saturating_sub(1));
        return Transition::pure(Model {
            nav: model.nav.clone().with_sidebar_cursor(cursor),
            ..model
        });
    }

    let viewport = model.viewport_rows;
    Transition::pure(Model {
        list: model.list.moved(delta).scrolled_into_view(viewport),
        ..model
    })
}

fn enter(model: Model) -> Transition {
    if model.nav.sidebar_focused {
        return match sidebar_rows().get(model.nav.sidebar_cursor) {
            Some(crate::model::SidebarRow::Item(resource)) => nav::go_to(
                Model {
                    nav: model.nav.clone().with_sidebar_focus(false),
                    ..model
                },
                resource,
            ),
            _ => Transition::pure(model),
        };
    }
    nav::open_selected(model)
}
