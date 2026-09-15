use serde_json::Value;

use crate::effect::Effect;
use crate::model::{Commit, Model, Picker};

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

    let effects = match &picker.commit {
        Commit::Batch {
            method,
            path,
            title,
        } => vec![Effect::Call {
            method: *method,
            path: path.clone(),
            query: Vec::new(),
            body: Some(Value::Array(chosen)),
            title: format!("{title}: {labels}"),
        }],
        Commit::Chain { .. } => Vec::new(),
        Commit::Replace {
            method,
            path,
            base,
            field,
            from,
            title,
        } => chosen
            .first()
            .and_then(|option| option.get(*from).cloned())
            .map(|value| Effect::Call {
                method: *method,
                path: path.clone(),
                query: Vec::new(),
                body: Some(kct_core::json::set_at(
                    base.clone(),
                    &format!("/{field}"),
                    value,
                )),
                title: format!("{title}: {labels}"),
            })
            .into_iter()
            .collect(),
        Commit::Each {
            method,
            base,
            key,
            title,
        } => chosen
            .iter()
            .filter_map(|option| option.get(*key).and_then(Value::as_str))
            .map(|id| Effect::Call {
                method: *method,
                path: base.child(id),
                query: Vec::new(),
                body: None,
                title: format!("{title}: {labels}"),
            })
            .collect(),
    };

    Transition::many(model.without_overlay(), effects)
}
