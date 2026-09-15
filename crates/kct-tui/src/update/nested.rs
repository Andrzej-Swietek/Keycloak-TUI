use std::sync::Arc;

use kct_core::Query;
use kct_core::catalog::Tab;

use crate::model::{List, Location, Model};
use crate::msg::Notice;

use super::nav::reload;
use super::transition::Transition;

/// Descend into a nested collection from the open detail.
#[must_use]
pub fn enter_nested(model: Model, tab: &Tab) -> Transition {
    let Tab::Nested {
        resource: child_id,
        slot,
        source,
        preset,
        ..
    } = tab
    else {
        return Transition::pure(model);
    };

    let Some(child) = kct_core::resources::by_id(child_id) else {
        return Transition::pure(
            model.noticed(Notice::failure(format!("unknown resource '{child_id}'"))),
        );
    };
    let Some(detail) = &model.detail else {
        return Transition::pure(model);
    };

    // A child that lives in the same document as its parent pins no slot: there is nothing
    // for the parent's identifier to fill in, and a singleton has no identifier to give.
    if slot.is_empty() {
        let crumb = Arc::clone(&detail.title);
        let context = model.context().clone();
        return descend(model, child, context, crumb);
    }

    // The slot value is usually the row's id, but some endpoints key off another field.
    let value = match source {
        Some(pointer) => kct_core::json::at(&detail.original, pointer)
            .map(|found| kct_core::json::to_line(found).into_owned()),
        None => Some(detail.id.clone()),
    };
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return Transition::pure(model.noticed(Notice::warning(format!(
            "cannot open {}: the parent has no {}",
            child.plural,
            source.unwrap_or("id")
        ))));
    };

    let context = preset
        .iter()
        .fold(model.context().clone(), |context, (name, pinned)| {
            context.with(name, *pinned)
        })
        .with(slot, value);

    let crumb = Arc::clone(&detail.title);
    descend(model, child, context, crumb)
}

fn descend(
    model: Model,
    child: &'static kct_core::catalog::Resource,
    context: kct_core::ports::Context,
    crumb: Arc<str>,
) -> Transition {
    reload(Model {
        nav: model.nav.pushed(Location {
            resource: child,
            context,
            crumb,
        }),
        list: List {
            query: Query::default(),
            ..List::default()
        },
        ..model
    })
}
