use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};

use crate::effect::Effect;
use crate::model::{Confirm, Model, Overlay, Prompt, Spotlight};

use super::{attributes, picker, prompt, spotlight_keys, transition::Transition};

/// Route a keypress to whichever modal is open.
#[must_use]
pub fn handle(model: Model, key: KeyEvent) -> Transition {
    let Some(overlay) = model.overlay.clone() else {
        return Transition::pure(model);
    };

    match overlay {
        Overlay::Spotlight(state) => spotlight_keys::handle(model, state, key),
        Overlay::Picker(state) => picker::handle(model, *state, key),
        Overlay::Attributes(state) => attributes::handle(model, state, key),
        Overlay::Prompt(state) => prompt::handle(model, state, key),
        Overlay::Confirm(state) => confirm(model, state, key),
        Overlay::Help { scroll } => {
            scrollable(model, key, scroll, |scroll| Overlay::Help { scroll })
        }
        Overlay::Viewer {
            title,
            body,
            scroll,
        } => {
            if matches!(key.code, KeyCode::Char('s')) {
                return super::tokens::save_viewer(model);
            }
            let copy = matches!(key.code, KeyCode::Char('y'));
            let text = body.to_string();
            let next = scrollable(model, key, scroll, move |scroll| Overlay::Viewer {
                title: Arc::clone(&title),
                body: Arc::clone(&body),
                scroll,
            });
            if copy {
                next.and(Effect::Copy {
                    text,
                    label: "contents".to_owned(),
                })
            } else {
                next
            }
        }
        Overlay::RequestLog { cursor } => {
            let last = model.log.len().saturating_sub(1);
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => Transition::pure(model.without_overlay()),
                KeyCode::Char('j') | KeyCode::Down => {
                    Transition::pure(model.with_overlay(Overlay::RequestLog {
                        cursor: cursor.saturating_add(1).min(last),
                    }))
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    Transition::pure(model.with_overlay(Overlay::RequestLog {
                        cursor: cursor.saturating_sub(1),
                    }))
                }
                _ => Transition::pure(model),
            }
        }
    }
}

fn confirm(model: Model, state: Confirm, key: KeyEvent) -> Transition {
    match key.code {
        KeyCode::Esc => Transition::pure(model.without_overlay()),

        KeyCode::Enter if state.is_satisfied() => {
            Transition::many(model.without_overlay(), state.on_confirm.as_ref().clone())
        }
        KeyCode::Enter => Transition::pure(model),

        // A yes/no confirmation also accepts the obvious single key.
        KeyCode::Char('y') if state.expect.is_none() => {
            Transition::many(model.without_overlay(), state.on_confirm.as_ref().clone())
        }
        KeyCode::Char('n') if state.expect.is_none() => Transition::pure(model.without_overlay()),

        _ if state.expect.is_some() => {
            Transition::pure(model.with_overlay(Overlay::Confirm(Confirm {
                typed: state.typed.apply(key),
                ..state
            })))
        }
        _ => Transition::pure(model),
    }
}

fn scrollable(
    model: Model,
    key: KeyEvent,
    scroll: u16,
    rebuild: impl Fn(u16) -> Overlay,
) -> Transition {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => Transition::pure(model.without_overlay()),
        KeyCode::Char('j') | KeyCode::Down => {
            Transition::pure(model.with_overlay(rebuild(scroll.saturating_add(1))))
        }
        KeyCode::Char('k') | KeyCode::Up => {
            Transition::pure(model.with_overlay(rebuild(scroll.saturating_sub(1))))
        }
        KeyCode::PageDown => {
            Transition::pure(model.with_overlay(rebuild(scroll.saturating_add(20))))
        }
        KeyCode::PageUp => Transition::pure(model.with_overlay(rebuild(scroll.saturating_sub(20)))),
        KeyCode::Home => Transition::pure(model.with_overlay(rebuild(0))),
        _ => Transition::pure(model),
    }
}

/// Re-rank palette results after the query changed.
#[must_use]
pub(super) fn researched(model: Model, state: Spotlight) -> Transition {
    Transition::pure(model.with_overlay(Overlay::Spotlight(state.researched())))
}

/// Keep a prompt's typed text.
#[must_use]
pub(super) fn typing(model: Model, state: Prompt, key: KeyEvent) -> Transition {
    Transition::pure(model.with_overlay(Overlay::Prompt(Prompt {
        input: state.input.apply(key),
        ..state
    })))
}
