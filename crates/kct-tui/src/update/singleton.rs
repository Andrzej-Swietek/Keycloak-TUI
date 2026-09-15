use crate::model::{Detail, Model};

use super::transition::Transition;

/// A singleton resource has nothing to pick from, so the object itself is what opens.
pub(super) fn open_singleton(model: Model) -> Transition {
    let resource = model.resource();
    let Some(object) = model.list.rows.first().cloned() else {
        return Transition::pure(model);
    };

    Transition::pure(Model {
        detail: Some(Detail {
            original: object.clone(),
            draft: object,
            loading: false,
            ..Detail::loading(String::new(), resource.singular, 0)
        }),
        ..model
    })
}
