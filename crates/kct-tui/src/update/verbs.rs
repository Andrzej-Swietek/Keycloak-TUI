//! Running the verb a key stands for on the screen in view.

use kct_core::catalog::{Ops, Verb};

use crate::model::Model;
use crate::msg::Notice;

use super::actions;
use super::transition::Transition;

pub(super) fn verb(model: Model, verb: Verb) -> Transition {
    let resource = model.resource();
    let needed = match verb {
        Verb::Create => Ops::CREATE,
        Verb::Delete => Ops::DELETE,
        _ => Ops::READ,
    };

    if !resource.ops.has(needed) {
        return Transition::pure(model.noticed(Notice::warning(format!(
            "{} cannot be {} here",
            resource.plural.to_lowercase(),
            if needed == Ops::CREATE {
                "created"
            } else {
                "deleted"
            }
        ))));
    }

    // Some screens create through a bespoke flow — an authorization policy's type is part of
    // its URL — so the key falls back to the action named for the verb.
    let named = match verb {
        Verb::Create => "create",
        Verb::Delete => "delete",
        _ => "",
    };
    let action = resource
        .actions
        .iter()
        .find(|action| action.verb == verb)
        .or_else(|| resource.actions.iter().find(|action| action.id == named));

    match action {
        Some(action) => actions::run(model, action),
        None => Transition::pure(model),
    }
}
