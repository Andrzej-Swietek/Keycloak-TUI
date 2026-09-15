use kct_core::catalog::FieldKind;
use serde_json::Value;

/// Interpret typed text as the field's native JSON type.
pub(super) fn parse(text: &str, kind: FieldKind) -> Value {
    match kind {
        FieldKind::Integer | FieldKind::Seconds => text
            .trim()
            .parse::<i64>()
            .map_or_else(|_| Value::String(text.to_owned()), Value::from),
        FieldKind::List => Value::Array(
            text.split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(|part| Value::String(part.to_owned()))
                .collect(),
        ),
        FieldKind::Json | FieldKind::Attributes => {
            serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_owned()))
        }
        _ => Value::String(text.to_owned()),
    }
}
