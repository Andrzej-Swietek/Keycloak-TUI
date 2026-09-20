//! Writing an open detail back to the server.
//!
//! Two shapes of write live here. Most objects have their own URL and take the whole draft, so
//! fields this build does not model travel back untouched. A few are elements inside a larger
//! document — the declarative user profile's attributes — and those put the whole document
//! back with just that element replaced, for the same reason.

use std::sync::Arc;

use kct_core::Method;
use kct_core::catalog::{Danger, Ops};
use kct_core::plan;

use crate::effect::Effect;
use crate::model::{Confirm, Detail, Model, Overlay};
use crate::msg::Notice;
use crate::text_input::TextInput;

use super::transition::Transition;
use super::warnings;

/// Save whatever detail is open, if any. Entry point for `:w`.
#[must_use]
pub(super) fn save_open(model: Model) -> Transition {
    match model.detail.clone() {
        Some(detail) => save(model, detail),
        None => Transition::pure(model.noticed(crate::msg::Notice::info("nothing open to save"))),
    }
}

/// Where a save is sent: the object's own URL, or the document it lives inside.
fn write_target(
    model: &Model,
    resource: &'static kct_core::catalog::Resource,
    id: &str,
) -> kct_core::Result<kct_core::plan::Planned> {
    if resource.embeds_items() {
        return Ok(kct_core::plan::Planned {
            path: resource.collection.resolve(&model.context().params)?,
            query: Vec::new(),
        });
    }
    plan::item(resource, model.context(), id)
}

pub(super) fn save(model: Model, detail: Detail) -> Transition {
    let resource = model.resource();
    if !resource.ops.has(Ops::UPDATE) {
        return Transition::pure(model.noticed(Notice::warning("this resource is read-only")));
    }
    if !detail.is_dirty() {
        return Transition::pure(model.noticed(Notice::info("nothing changed")));
    }

    let id = if detail.id.is_empty() {
        model.session.realm.to_string()
    } else {
        detail.id.clone()
    };
    let planned = match write_target(&model, resource, &id) {
        Ok(planned) => planned,
        Err(error) => {
            return Transition::pure(model.noticed(Notice::from_error("cannot save", &error)));
        }
    };

    let body = resource
        .with_item(
            (*model.list.envelope).clone(),
            &id,
            Some(detail.draft.clone()),
        )
        .unwrap_or_else(|| detail.draft.clone());

    let effect = Effect::call(
        Method::Put,
        planned.path,
        Some(body),
        format!(
            "saved {} '{}'",
            resource.singular.to_lowercase(),
            detail.title
        ),
    );

    let changes = detail.pending_changes();
    let body = warnings::before_save(&model, &changes)
        .into_iter()
        .map(|warning| format!("⚠ {warning}\n\n"))
        .chain(std::iter::once(kct_core::json::pretty(&changes)))
        .collect::<String>();

    Transition::pure(model.with_overlay(Overlay::Confirm(Confirm {
        title: Arc::from("Save changes"),
        body: Arc::from(body),
        danger: Danger::Confirm,
        expect: None,
        typed: TextInput::default(),
        on_confirm: Arc::new(vec![effect]),
    })))
}
