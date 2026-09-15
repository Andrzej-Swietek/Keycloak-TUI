use serde_json::Value;

use crate::catalog::{Encoding, Field, FieldKind, Resource, Tab};
use crate::json;

use super::finding::Finding;

/// Report form controls whose declared [`FieldKind`] does not fit the sampled values.
///
/// Yields nothing without samples, and nothing for a field whose [`Encoding`] already
/// declares how the value is wrapped. Findings are [`super::Severity::Missing`]: the value stays
/// reachable through the raw tab.
#[must_use]
pub fn compare(resource: &Resource, samples: &[Value]) -> Vec<Finding> {
    if samples.is_empty() {
        return Vec::new();
    }

    fields(resource)
        .filter_map(|field| {
            let seen: Vec<&Value> = samples
                .iter()
                .filter_map(|sample| json::at(sample, field.pointer))
                .filter(|value| !value.is_null())
                .collect();
            let observed = *seen.first()?;
            let better = stringly_boolean(field, &seen).or_else(|| suggest(field, observed))?;
            Some(Finding::missing(
                resource.id,
                format!(
                    "'{}' is declared {:?} but the server sends {} — {better} would edit it",
                    field.pointer,
                    field.kind,
                    describe(observed),
                ),
            ))
        })
        .collect()
}

fn fields(resource: &Resource) -> impl Iterator<Item = &'static Field> {
    resource
        .tabs
        .iter()
        .filter_map(|tab| match tab {
            Tab::Form { fields, .. } => Some(*fields),
            _ => None,
        })
        .flatten()
}

/// A text field whose every value is `"true"` or `"false"`.
///
/// Keycloak stores plenty of booleans as strings — SMTP switches, identity-provider config —
/// and a text box makes the operator type the word and get the spelling right. Declaring the
/// field `Bool` with a stringly encoding gives a toggle and writes the same JSON.
fn stringly_boolean(field: &Field, seen: &[&Value]) -> Option<&'static str> {
    if !matches!(field.kind, FieldKind::Text | FieldKind::Multiline) {
        return None;
    }
    seen.iter()
        .all(|value| matches!(value.as_str(), Some("true" | "false")))
        .then_some("a stringly Bool")
}

/// The control that would fit the observed value better, if any.
fn suggest(field: &Field, observed: &Value) -> Option<&'static str> {
    let declared = field.kind;
    let shape = Shape::of(observed)?;

    // A field that already declares how the value is wrapped on the wire is not mismatched:
    // component config really is `["true"]` for a boolean, and the encoding says so.
    if !matches!(field.encoding, Encoding::Native) {
        return None;
    }
    let fits = matches!(
        (declared, shape),
        (FieldKind::StringMap, Shape::StringMap)
            | (FieldKind::Attributes, Shape::ListMap)
            | (FieldKind::List, Shape::StringList)
            | (FieldKind::Bool, Shape::Bool)
            | (FieldKind::Integer | FieldKind::Seconds, Shape::Number)
            | (FieldKind::Json, Shape::Nested)
    );
    if fits || matches!(declared, FieldKind::ReadOnly | FieldKind::Secret) {
        return None;
    }

    // A scalar in a text box is the normal case and needs no advice.
    if matches!(shape, Shape::Text) {
        return None;
    }

    Some(match shape {
        Shape::StringMap => "StringMap",
        Shape::ListMap => "Attributes",
        Shape::StringList => "List",
        Shape::Bool => "Bool",
        Shape::Number => "Integer",
        Shape::Nested | Shape::Text => return None,
    })
}

#[derive(Clone, Copy)]
enum Shape {
    StringMap,
    ListMap,
    StringList,
    Bool,
    Number,
    Text,
    Nested,
}

impl Shape {
    fn of(value: &Value) -> Option<Self> {
        Some(match value {
            Value::Bool(_) => Self::Bool,
            Value::Number(_) => Self::Number,
            Value::String(_) => Self::Text,
            Value::Array(items) if items.is_empty() => return None,
            Value::Array(items) if items.iter().all(Value::is_string) => Self::StringList,
            Value::Object(map) if map.is_empty() => return None,
            Value::Object(map) if map.values().all(Value::is_string) => Self::StringMap,
            Value::Object(map) if map.values().all(is_string_list) => Self::ListMap,
            _ => Self::Nested,
        })
    }
}

fn is_string_list(value: &Value) -> bool {
    value
        .as_array()
        .is_some_and(|items| !items.is_empty() && items.iter().all(Value::is_string))
}

fn describe(value: &Value) -> &'static str {
    match Shape::of(value) {
        Some(Shape::StringMap) => "a flat string map",
        Some(Shape::ListMap) => "a map of string lists",
        Some(Shape::StringList) => "a list of strings",
        Some(Shape::Bool) => "a boolean",
        Some(Shape::Number) => "a number",
        Some(Shape::Text) => "a string",
        _ => "nested JSON",
    }
}
