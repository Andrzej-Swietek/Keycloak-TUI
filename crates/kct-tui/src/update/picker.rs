use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use kct_core::catalog::Shape;
use serde_json::Value;

use crate::effect::Effect;
use crate::model::{Model, Overlay, Picker};
use crate::msg::{Fetch, Notice};

use super::commit::commit;
use super::{pick_flow, pick_groups, pick_roles, pick_specs, transition::Transition};

/// Open a picker for one of the actions that grant or revoke something.
#[must_use]
pub fn open(model: Model, action: &'static str) -> Transition {
    open_with(model, action, &pick_roles::Extras::new())
}

/// Open a picker carrying slots chosen in an earlier step.
#[must_use]
pub fn open_with(model: Model, action: &'static str, extras: &pick_roles::Extras) -> Transition {
    let spec = match action {
        "assign-role" => pick_specs::assign_role(&model, false),
        "unassign-role" => pick_specs::assign_role(&model, true),
        "join-group" => pick_specs::join_group(&model),
        "add-client-scope" => pick_specs::add_client_scope(&model),
        "pick-client-for-role" => pick_roles::pick_client(&model),
        "assign-client-role" => pick_roles::assign_client_role(&model, extras),
        "add-composite" => pick_roles::add_composite(&model),
        "set-execution-requirement" => pick_roles::set_requirement(&model),
        "add-execution" => pick_flow::add_execution(&model),
        "move-group" => pick_groups::move_group(&model),
        _ => return Transition::pure(model),
    };

    let mut spec = match spec {
        Ok(spec) => spec,
        Err(error) => {
            return Transition::pure(model.noticed(Notice::from_error("cannot open", &error)));
        }
    };

    let (model, token) = model.issue_token();

    let require = std::mem::take(&mut spec.require);
    let without: Arc<[String]> = Arc::from(std::mem::take(&mut spec.without));
    let inline = spec.options.take();
    let load = |fetch, path| Effect::Load {
        token,
        fetch,
        path,
        query: Vec::new(),
        shape: Shape::Array,
    };
    // Candidates already in hand need no round trip.
    if let Some(options) = inline {
        let picker = Picker::loading(spec.title, spec.label, spec.commit, token)
            .requiring(require)
            .with_exclusions(without)
            .with_options(Arc::from(options));
        return Transition::pure(model.with_overlay(Overlay::picker(picker)));
    }

    let effects = std::iter::once(load(Fetch::Options, spec.source))
        .chain(
            spec.exclude
                .into_iter()
                .map(|path| load(Fetch::Exclusions, path)),
        )
        .collect();

    Transition::many(
        model.with_overlay(Overlay::picker(
            Picker::loading(spec.title, spec.label, spec.commit, token)
                .requiring(require)
                .with_exclusions(without),
        )),
        effects,
    )
}

/// Candidates that are already attached and must not be offered again.
#[must_use]
pub fn excluded(
    model: Model,
    token: u64,
    result: Result<Arc<[Value]>, kct_core::Error>,
) -> Transition {
    let Some(Overlay::Picker(picker)) = model.overlay.clone() else {
        return Transition::pure(model);
    };
    if picker.token != token {
        return Transition::pure(model);
    }

    // Several sources can contribute, so exclusions accumulate rather than replace.
    let already_taken: Vec<String> = picker
        .excluded
        .iter()
        .cloned()
        .chain(
            result
                .unwrap_or_default()
                .iter()
                .filter_map(|row| row.get(picker.exclude_key).and_then(Value::as_str))
                .map(ToOwned::to_owned),
        )
        .collect();

    Transition::pure(model.with_overlay(Overlay::picker(
        picker.with_exclusions(Arc::from(already_taken)),
    )))
}

/// Candidates arrived. A stale reply for a picker that is gone is dropped.
#[must_use]
pub fn loaded(
    model: Model,
    token: u64,
    result: Result<Arc<[Value]>, kct_core::Error>,
) -> Transition {
    let Some(Overlay::Picker(picker)) = model.overlay.clone() else {
        return Transition::pure(model);
    };
    if picker.token != token {
        return Transition::pure(model);
    }

    match result {
        Ok(options) => {
            Transition::pure(model.with_overlay(Overlay::picker(picker.with_options(options))))
        }
        Err(error) => Transition::pure(
            model
                .without_overlay()
                .noticed(Notice::from_error("cannot list candidates", &error)),
        ),
    }
}

#[must_use]
pub fn handle(model: Model, picker: Picker, key: KeyEvent) -> Transition {
    match key.code {
        KeyCode::Esc => Transition::pure(model.without_overlay()),
        KeyCode::Down => Transition::pure(model.with_overlay(Overlay::picker(picker.moved(1)))),
        KeyCode::Up => Transition::pure(model.with_overlay(Overlay::picker(picker.moved(-1)))),
        KeyCode::Tab => {
            Transition::pure(model.with_overlay(Overlay::picker(picker.toggled().moved(1))))
        }
        KeyCode::Enter => commit(model, &picker),
        _ => Transition::pure(
            model.with_overlay(Overlay::picker(
                Picker {
                    input: picker.input.apply(key),
                    ..picker
                }
                .refiltered(),
            )),
        ),
    }
}
