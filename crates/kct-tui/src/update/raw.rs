use kct_core::json;
use serde_json::Value;

use crate::model::Detail;

/// The raw tab's fields: one per JSON leaf, labelled and addressed by its pointer.
///
/// Every value in the representation gets a field, including those no form models.
#[must_use]
pub fn leaves(detail: &Detail) -> Vec<(String, Value)> {
    json::leaves(&detail.draft)
        .into_iter()
        .map(|(pointer, value)| (pointer, value.clone()))
        .collect()
}

/// Parse typed text back into the JSON type the leaf already had, so editing `enabled` keeps
/// it a boolean rather than turning it into the string `"false"`.
#[must_use]
pub fn reparse(previous: &Value, text: &str) -> Value {
    match previous {
        Value::Bool(_) => match text.trim() {
            "true" | "yes" => Value::Bool(true),
            "false" | "no" => Value::Bool(false),
            other => Value::String(other.to_owned()),
        },
        Value::Number(_) => text
            .trim()
            .parse::<i64>()
            .map_or_else(|_| Value::String(text.to_owned()), Value::from),
        Value::Null if text.is_empty() => Value::Null,
        _ => Value::String(text.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn every_value_including_unknown_ones_is_addressable() {
        let detail = Detail {
            draft: json!({
                "enabled": true,
                "aFieldFromTheFuture": {"nested": ["x", "y"]}
            }),
            ..Detail::loading("id", "title", 1)
        };

        let pointers: Vec<String> = leaves(&detail).into_iter().map(|(p, _)| p).collect();
        assert!(pointers.contains(&"/enabled".to_owned()));
        assert!(pointers.contains(&"/aFieldFromTheFuture/nested/0".to_owned()));
        assert!(pointers.contains(&"/aFieldFromTheFuture/nested/1".to_owned()));
    }

    #[test]
    fn editing_keeps_the_json_type_the_server_used() {
        assert_eq!(reparse(&json!(true), "false"), json!(false));
        assert_eq!(reparse(&json!(300), "600"), json!(600));
        assert_eq!(reparse(&json!("a"), "b"), json!("b"));
        // A number field typed as prose stays text rather than silently becoming 0.
        assert_eq!(reparse(&json!(300), "soon"), json!("soon"));
    }
}
