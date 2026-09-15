use crossterm::event::{KeyCode, KeyEvent};
use kct_core::resources;

use crate::effect::Effect;
use crate::model::{Model, Overlay, Spotlight};
use crate::msg::Notice;
use crate::spotlight::{Command, Ex, Global};

use super::{actions, commands, nav, overlay, tokens, transition::Transition};

#[must_use]
pub fn handle(model: Model, state: Spotlight, key: KeyEvent) -> Transition {
    match key.code {
        KeyCode::Esc => Transition::pure(model.without_overlay()),
        KeyCode::Down | KeyCode::Tab => {
            Transition::pure(model.with_overlay(Overlay::Spotlight(state.moved(1))))
        }
        KeyCode::Up | KeyCode::BackTab => {
            Transition::pure(model.with_overlay(Overlay::Spotlight(state.moved(-1))))
        }
        KeyCode::Enter if state.ex.is_some() => {
            run_ex(model.without_overlay(), state.ex.expect("checked"))
        }
        KeyCode::Enter => match state.selected() {
            Some(entry) if !entry.available => {
                let why = entry.note.clone().unwrap_or_default();
                Transition::pure(model.noticed(Notice::warning(why)))
            }
            Some(entry) => run(model.without_overlay(), entry.command.clone()),
            None => Transition::pure(model),
        },
        _ => overlay::researched(
            model,
            Spotlight {
                input: state.input.apply(key),
                ..state
            },
        ),
    }
}

/// Vim commands act on the open object, which is where a `:w` habit points.
fn run_ex(model: Model, ex: Ex) -> Transition {
    match ex {
        Ex::Write => super::save::save_open(model),
        Ex::WriteQuit => super::save::save_open(model.closing_after_save()),
        Ex::Quit => {
            if model
                .detail
                .as_ref()
                .is_some_and(crate::model::Detail::is_dirty)
            {
                Transition::pure(model.noticed(Notice::warning(
                    "unsaved changes — :w to save, :q! to discard",
                )))
            } else {
                nav::back(model)
            }
        }
        Ex::QuitForce => nav::back(model),
        Ex::Edit => nav::reload(model),
        Ex::QuitAll => Transition::with(
            Model {
                quit: true,
                ..model
            },
            Effect::Quit,
        ),
    }
}

fn run(model: Model, command: Command) -> Transition {
    match command {
        Command::Open(id) => match resources::by_id(id) {
            Some(resource) => nav::go_to(model, resource),
            None => Transition::pure(model),
        },
        Command::Realm(realm) => nav::switch_realm(model, realm),
        Command::Run { resource, action } => run_action(model, resource, action),
        Command::Global(global) => run_global(model, global),
    }
}

/// Running another screen's action navigates there first, so the operator ends up looking at
/// what they just changed.
fn run_action(model: Model, resource_id: &'static str, action_id: &'static str) -> Transition {
    let Some(resource) = resources::by_id(resource_id) else {
        return Transition::pure(model);
    };
    let Some(action) = resource
        .actions
        .iter()
        .find(|action| action.id == action_id)
    else {
        return Transition::pure(model);
    };

    if model.resource().id == resource_id {
        return actions::run(model, action);
    }

    let moved = nav::go_to(model, resource);
    let started = actions::run(moved.model, action);
    Transition::many(
        started.model,
        moved.effects.into_iter().chain(started.effects).collect(),
    )
}

fn run_global(model: Model, global: Global) -> Transition {
    match global {
        Global::Help => Transition::pure(model.with_overlay(Overlay::Help { scroll: 0 })),
        Global::Refresh => nav::reload(model),
        Global::Back => nav::back(model),
        Global::Quit => Transition::with(
            Model {
                quit: true,
                ..model
            },
            Effect::Quit,
        ),
        Global::SwitchRealm => commands::open_palette(model),
        Global::ServerInfo => match resources::by_id("server-info") {
            Some(resource) => nav::go_to(model, resource),
            None => Transition::pure(model),
        },
        Global::RequestLog => {
            Transition::pure(model.with_overlay(Overlay::RequestLog { cursor: 0 }))
        }
        Global::ToggleMascot => {
            let prefs = model.prefs.next_mascot(model.mascots.found.len());
            let said = prefs.mascot.map_or_else(
                || "mascot hidden".to_owned(),
                |who| format!("mascot: {}", crate::view::mascot_name(&model, who)),
            );
            Transition::pure(Model { prefs, ..model }.noticed(Notice::info(said)))
        }
        Global::ReloadMascots => Transition::with(model, crate::effect::Effect::LoadMascots),
        Global::ToggleDryRun => {
            let dry_run = !model.session.dry_run;
            let model = Model {
                session: crate::model::Session {
                    dry_run,
                    ..model.session
                },
                ..model
            };
            Transition::pure(model.noticed(if dry_run {
                Notice::warning("dry-run on: writes are suppressed")
            } else {
                Notice::info("dry-run off: writes will be sent")
            }))
        }
        Global::ReAuthenticate => Transition::with(
            model,
            Effect::DetectCapabilities {
                realm: String::new(),
            },
        ),
        Global::InspectToken => tokens::inspect(model),
        Global::CopyAccessToken => tokens::copy_token(model),
        Global::CopyAsCurl => tokens::copy_curl(model),
        Global::RealmJwks => tokens::realm_keys(model),
    }
}
