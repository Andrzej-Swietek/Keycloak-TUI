//! Creating an object from what the operator typed: a name, or a whole representation.

use kct_core::{Method, plan, resources};

use crate::effect::Effect;
use crate::model::Model;
use crate::msg::Notice;

use super::transition::Transition;

/// Create an object from a pasted representation.
///
/// The text is sent as the body verbatim; the server reports what it does not accept.
pub(super) fn create_from_json(model: Model, resource_id: &'static str, body: &str) -> Transition {
    let Some(resource) = resources::by_id(resource_id) else {
        return Transition::pure(model);
    };
    let parsed = match serde_json::from_str::<serde_json::Value>(body) {
        Ok(parsed) => parsed,
        Err(why) => {
            return Transition::pure(
                model.noticed(Notice::failure(format!("that is not JSON: {why}"))),
            );
        }
    };

    match plan::list(resource, model.context(), &model.list.query) {
        Ok(planned) => Transition::with(
            model,
            Effect::call(
                Method::Post,
                planned.path,
                Some(parsed),
                format!("created {}", resource.singular.to_lowercase()),
            ),
        ),
        Err(error) => Transition::pure(model.noticed(Notice::from_error("cannot create", &error))),
    }
}
