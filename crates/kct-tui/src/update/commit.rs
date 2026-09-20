use serde_json::Value;

use crate::model::{Commit, Model, Overlay, Picker};

use super::pick_roles;
use super::picker::open_with;
use super::transition::Transition;

/// Turn what was picked into requests.
pub(super) fn commit(model: Model, picker: &Picker) -> Transition {
    let chosen = picker.picked();
    if chosen.is_empty() {
        return Transition::pure(model.without_overlay());
    }

    let labels = chosen
        .iter()
        .map(|option| picker.label_of(option))
        .collect::<Vec<_>>()
        .join(", ");

    // A chaining step writes nothing; it hands the choice to the next picker.
    if let Commit::Chain { next, slot, key } = &picker.commit {
        let Some(id) = chosen
            .first()
            .and_then(|option| option.get(*key))
            .and_then(Value::as_str)
        else {
            return Transition::pure(model.without_overlay());
        };
        let mut extras = pick_roles::Extras::new();
        extras.insert(slot, id.to_owned());
        return open_with(model.without_overlay(), next, &extras);
    }

    // Naming step: the choice settles the path, the operator settles the name.
    if let Commit::Name {
        base,
        from,
        title,
        singular,
    } = &picker.commit
    {
        let Some(kind) = chosen
            .first()
            .and_then(|option| option.get(*from))
            .and_then(Value::as_str)
        else {
            return Transition::pure(model.without_overlay());
        };
        return Transition::pure(model.without_overlay().with_overlay(Overlay::Prompt(
            crate::model::Prompt {
                title: std::sync::Arc::from(*title),
                help: std::sync::Arc::from(format!("Type: {kind}")),
                input: crate::text_input::TextInput::default(),
                secret: false,
                ask: crate::model::Ask::PostNamed {
                    path: base.sub(kind),
                    field: "name",
                    singular,
                },
            },
        )));
    }

    let effects = super::requests::requests(&picker.commit, &chosen, &labels);

    Transition::many(model.without_overlay(), effects)
}
