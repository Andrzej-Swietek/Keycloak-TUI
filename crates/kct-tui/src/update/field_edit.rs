use crossterm::event::{KeyCode, KeyEvent};
use kct_core::catalog::{FieldKind, Ops};
use serde_json::Value;

use crate::model::{Detail, FieldEdit, Model};
use crate::msg::Notice;
use crate::text_input::TextInput;

use super::detail::visible_fields;
use super::parse::parse;
use super::raw;
use super::transition::Transition;

pub(super) fn begin_edit(model: Model, detail: Detail) -> Transition {
    let resource = model.resource();
    let Some(tab) = resource.tabs.get(detail.tab) else {
        return Transition::pure(model);
    };

    if matches!(tab, kct_core::catalog::Tab::Raw) {
        return begin_raw_edit(model, detail);
    }
    let fields = visible_fields(&model, tab);
    let Some(field) = fields.get(detail.cursor) else {
        return Transition::pure(model);
    };

    if !resource.ops.has(Ops::UPDATE) || matches!(field.kind, FieldKind::ReadOnly) {
        return Transition::pure(
            model.noticed(Notice::info(format!("{} is read-only", field.label))),
        );
    }

    if matches!(field.kind, FieldKind::Attributes | FieldKind::StringMap) {
        return super::attributes::open(model.clone(), &detail, field);
    }

    // A checkbox does not need a text editor; toggling in place is what an operator expects.
    if matches!(field.kind, FieldKind::Bool) {
        let current = field
            .read(&detail.draft)
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        return Transition::pure(Model {
            detail: Some(Detail {
                draft: field.write(detail.draft.clone(), Value::Bool(!current)),
                ..detail
            }),
            ..model
        });
    }

    let current = field
        .read(&detail.draft)
        .map(|value| kct_core::json::to_line(&value).into_owned())
        .unwrap_or_default();

    let suggestions = super::suggest::for_field(field, model.resource(), &model.session);

    Transition::pure(Model {
        detail: Some(Detail {
            editing: Some(FieldEdit::new(
                detail.cursor,
                TextInput::new(current),
                suggestions,
            )),
            ..detail
        }),
        ..model
    })
}

/// Start editing whichever JSON leaf is highlighted on the raw tab.
fn begin_raw_edit(model: Model, detail: Detail) -> Transition {
    if !model.resource().ops.has(Ops::UPDATE) {
        return Transition::pure(model.noticed(Notice::info("this resource is read-only")));
    }
    let Some((_, value)) = raw::leaves(&detail).into_iter().nth(detail.cursor) else {
        return Transition::pure(model);
    };

    Transition::pure(Model {
        detail: Some(Detail {
            editing: Some(FieldEdit::new(
                detail.cursor,
                TextInput::new(kct_core::json::to_line(&value).into_owned()),
                Vec::new(),
            )),
            ..detail
        }),
        ..model
    })
}

fn commit_raw(detail: &Detail, index: usize, text: &str) -> Value {
    match raw::leaves(detail).into_iter().nth(index) {
        Some((pointer, previous)) => kct_core::json::set_at(
            detail.draft.clone(),
            &pointer,
            raw::reparse(&previous, text),
        ),
        None => detail.draft.clone(),
    }
}

pub(super) fn edit(model: Model, detail: Detail, key: KeyEvent) -> Transition {
    let Some(editing) = detail.editing.clone() else {
        return Transition::pure(model);
    };

    match key.code {
        KeyCode::Esc => Transition::pure(Model {
            detail: Some(Detail {
                editing: None,
                ..detail
            }),
            ..model
        }),
        KeyCode::Enter => {
            let tab = model.resource().tabs.get(detail.tab);
            let draft = match tab {
                Some(kct_core::catalog::Tab::Raw) => {
                    commit_raw(&detail, editing.field, &editing.value())
                }
                Some(tab) => match visible_fields(&model, tab).get(editing.field).copied() {
                    // `value()`, not the raw input: a highlighted suggestion is the choice, and
                    // committing what was typed under it silently discards the pick.
                    Some(field) => {
                        field.write(detail.draft.clone(), parse(&editing.value(), field.kind))
                    }
                    None => detail.draft.clone(),
                },
                None => detail.draft.clone(),
            };

            Transition::pure(Model {
                detail: Some(Detail {
                    draft,
                    editing: None,
                    ..detail
                }),
                ..model
            })
        }
        // Tab and the arrows walk the suggestions; typing filters them.
        KeyCode::Tab | KeyCode::Down => Transition::pure(Model {
            detail: Some(Detail {
                editing: Some(editing.moved(1)),
                ..detail
            }),
            ..model
        }),
        KeyCode::BackTab | KeyCode::Up => Transition::pure(Model {
            detail: Some(Detail {
                editing: Some(editing.moved(-1)),
                ..detail
            }),
            ..model
        }),
        _ => Transition::pure(Model {
            detail: Some(Detail {
                editing: Some(
                    FieldEdit {
                        input: editing.input.clone().apply(key),
                        ..editing
                    }
                    .refiltered(),
                ),
                ..detail
            }),
            ..model
        }),
    }
}
