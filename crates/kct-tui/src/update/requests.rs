//! Turning a confirmed selection into requests.

use serde_json::Value;

use crate::effect::Effect;
use crate::model::Commit;

/// The requests one confirmed selection turns into.
pub(super) fn requests(commit: &Commit, chosen: &[Value], labels: &str) -> Vec<Effect> {
    match commit {
        Commit::Batch {
            method,
            path,
            title,
        } => vec![Effect::call(
            *method,
            path.clone(),
            Some(Value::Array(chosen.to_vec())),
            format!("{title}: {labels}"),
        )],
        Commit::Chain { .. } | Commit::Name { .. } => Vec::new(),
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
            .map(|value| {
                Effect::call(
                    *method,
                    path.clone(),
                    Some(kct_core::json::set_at(
                        base.clone(),
                        &format!("/{field}"),
                        value,
                    )),
                    format!("{title}: {labels}"),
                )
            })
            .into_iter()
            .collect(),
        Commit::EachBody {
            method,
            path,
            field,
            from,
            title,
        } => chosen
            .iter()
            .filter_map(|option| option.get(*from).cloned())
            .map(|value| {
                Effect::call(
                    *method,
                    path.clone(),
                    Some(kct_core::json::set_at(
                        Value::Object(serde_json::Map::new()),
                        &format!("/{field}"),
                        value,
                    )),
                    format!("{title}: {labels}"),
                )
            })
            .collect(),
        Commit::EachUnder {
            method,
            base,
            key,
            suffix,
            body,
            title,
        } => chosen
            .iter()
            .filter_map(|option| option.get(*key).and_then(Value::as_str))
            .map(|id| {
                Effect::call(
                    *method,
                    base.child(id).sub(suffix),
                    Some(body.clone()),
                    format!("{title}: {labels}"),
                )
            })
            .collect(),
        Commit::Each {
            method,
            base,
            key,
            title,
        } => chosen
            .iter()
            .filter_map(|option| option.get(*key).and_then(Value::as_str))
            .map(|id| Effect::call(*method, base.child(id), None, format!("{title}: {labels}")))
            .collect(),
    }
}
