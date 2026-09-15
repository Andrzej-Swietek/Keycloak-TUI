use serde_json::Value;

use crate::json;

use super::field::FieldKind;

/// How a field's value is *stored*, independent of how it is edited.
///
/// Keycloak writes what an operator experiences as one checkbox three ways: a JSON boolean
/// on a representation, `"true"` inside a client's `attributes`, and `["true"]` inside a
/// component's `config`. Writing the wrong one is accepted and has no effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Native,
    /// A JSON string, even for booleans and numbers (`client.attributes`).
    Stringly,
    /// A single-element array of strings (`component.config`).
    ConfigList,
}

/// Turn a stored string back into the native value the field kind implies.
pub(super) fn decode_text(text: &str, kind: FieldKind) -> Value {
    match kind {
        FieldKind::Bool => match text {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            other => Value::String(other.to_owned()),
        },
        FieldKind::Integer | FieldKind::Seconds => text
            .parse::<i64>()
            .map_or_else(|_| Value::String(text.to_owned()), Value::from),
        _ => Value::String(text.to_owned()),
    }
}

/// Render a native value as the string the server stores.
///
/// Deliberately not `json::to_line`: that renders booleans for humans ("yes"/"no"), which as
/// a stored value Keycloak silently ignores.
pub(super) fn encode_text(value: &Value) -> Value {
    Value::String(match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        other => json::compact(other),
    })
}

/// `1800` -> `30m`. Long token lifespans are unreadable as raw seconds.
#[must_use]
pub fn format_seconds(total: i64) -> String {
    if total == 0 {
        return "0".to_owned();
    }
    let (sign, seconds) = if total < 0 {
        ("-", -total)
    } else {
        ("", total)
    };
    let rendered = [
        (seconds / 86_400, "d"),
        ((seconds % 86_400) / 3_600, "h"),
        ((seconds % 3_600) / 60, "m"),
        (seconds % 60, "s"),
    ]
    .iter()
    .filter(|(amount, _)| *amount > 0)
    .map(|(amount, unit)| format!("{amount}{unit}"))
    .collect::<Vec<_>>()
    .join(" ");

    format!("{sign}{rendered}")
}
