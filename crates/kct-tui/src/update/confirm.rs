use std::sync::Arc;

use kct_core::Result;
use kct_core::catalog::{Action, Danger};

use crate::effect::Effect;
use crate::model::{Confirm, Model, Overlay};
use crate::msg::Notice;
use crate::text_input::TextInput;

use super::transition::Transition;

/// Run an effect, asking first unless the action is harmless.
#[must_use]
pub(super) fn ask(
    model: Model,
    action: &'static Action,
    question: &str,
    effect: Effect,
) -> Transition {
    if action.danger == Danger::Safe {
        return Transition::with(model, effect);
    }

    Transition::pure(model.with_overlay(Overlay::Confirm(Confirm {
        title: Arc::from(action.title),
        body: Arc::from(question),
        danger: action.danger,
        expect: None,
        typed: TextInput::default(),
        on_confirm: Arc::new(vec![effect]),
    })))
}

/// Build an effect, or report the planning failure as a notice and ask nothing.
#[must_use]
pub(super) fn planned(
    model: Model,
    action: &'static Action,
    question: &str,
    plan: impl Fn(&Model) -> Result<Effect>,
) -> Transition {
    match plan(&model) {
        Ok(effect) => ask(model, action, question, effect),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(action.title, &error))),
    }
}
