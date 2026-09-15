use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use kct_core::catalog::FieldKind;

use crate::model::{AttributeColumn, AttributeShape, Attributes, Detail, Model, Overlay};
use crate::msg::Notice;
use crate::text_input::TextInput;

use super::transition::Transition;

/// Open the key–value editor for the field under the cursor.
#[must_use]
pub fn open(model: Model, detail: &Detail, field: &kct_core::catalog::Field) -> Transition {
    let shape = match field.kind {
        FieldKind::StringMap => AttributeShape::StringValued,
        _ => AttributeShape::ListValued,
    };

    let editor = Attributes::open(
        field.label,
        field.pointer,
        shape,
        kct_core::json::at(&detail.draft, field.pointer),
        Arc::clone(&model.session.declared_attributes),
        model.session.unmanaged_attributes == crate::model::UnmanagedAttributes::Discarded,
    );

    Transition::pure(model.with_overlay(Overlay::Attributes(editor)))
}

#[must_use]
pub fn handle(model: Model, editor: Attributes, key: KeyEvent) -> Transition {
    if let Some(input) = editor.editing.clone() {
        return editing(model, editor, input, key);
    }

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => close(model, &editor),
        KeyCode::Char('j') | KeyCode::Down => show(model, editor.moved(1)),
        KeyCode::Char('k') | KeyCode::Up => show(model, editor.moved(-1)),
        KeyCode::Tab | KeyCode::Char('h' | 'l') | KeyCode::Left | KeyCode::Right => {
            show(model, editor.switched())
        }
        KeyCode::Char('a') => show(model, editor.added()),
        KeyCode::Char('d') => show(model, editor.removed()),
        KeyCode::Enter => {
            let current = editor
                .rows
                .get(editor.cursor)
                .map(|(key, value)| match editor.column {
                    AttributeColumn::Key => key.clone(),
                    AttributeColumn::Value => value.clone(),
                })
                .unwrap_or_default();

            show(
                model,
                Attributes {
                    editing: Some(TextInput::new(current)),
                    ..editor
                },
            )
        }
        _ => show(model, editor),
    }
}

fn editing(model: Model, editor: Attributes, input: TextInput, key: KeyEvent) -> Transition {
    match key.code {
        KeyCode::Esc => show(
            model,
            Attributes {
                editing: None,
                ..editor
            },
        ),
        KeyCode::Enter => {
            let text = input.value().to_owned();
            show(model, editor.committed(&text))
        }
        _ => show(
            model,
            Attributes {
                editing: Some(input.apply(key)),
                ..editor
            },
        ),
    }
}

/// Closing writes the map back into the draft; the object itself still needs saving.
fn close(model: Model, editor: &Attributes) -> Transition {
    let Some(detail) = model.detail.clone() else {
        return Transition::pure(model.without_overlay());
    };

    let draft = kct_core::json::set_at(detail.draft.clone(), editor.pointer, editor.to_json());
    let undeclared = editor.undeclared().join(", ");

    let model = Model {
        detail: Some(Detail { draft, ..detail }),
        ..model
    }
    .without_overlay();

    if undeclared.is_empty() {
        return Transition::pure(model);
    }

    Transition::pure(model.noticed(Notice::warning(format!(
        "{undeclared} are not declared in this realm's user profile; a save will silently \
         drop them"
    ))))
}

fn show(model: Model, editor: Attributes) -> Transition {
    Transition::pure(model.with_overlay(Overlay::Attributes(editor)))
}
