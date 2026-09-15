use serde_json::{Map, Value};

use super::pointer;

/// RFC 7386 JSON Merge Patch, applied functionally.
///
/// `null` deletes a key, objects merge recursively, everything else replaces. Arrays replace
/// wholesale, which matches how Keycloak treats `redirectUris` and attribute list values.
#[must_use]
pub fn merge(base: Value, patch: Value) -> Value {
    match (base, patch) {
        (Value::Object(base), Value::Object(patch)) => {
            Value::Object(patch.into_iter().fold(base, |mut acc, (key, value)| {
                if value.is_null() {
                    acc.remove(&key);
                } else {
                    let previous = acc.remove(&key).unwrap_or(Value::Null);
                    acc.insert(key, merge(previous, value));
                }
                acc
            }))
        }
        (_, patch) => patch,
    }
}

/// The merge patch that turns `from` into `to`.
///
/// Used to show exactly what a save will change, and to keep payloads minimal.
#[must_use]
pub fn diff(from: &Value, to: &Value) -> Value {
    match (from, to) {
        (Value::Object(before), Value::Object(after)) => {
            let changed = after.iter().filter_map(|(key, new)| match before.get(key) {
                Some(old) if old == new => None,
                Some(old) => Some((key.clone(), diff(old, new))),
                None => Some((key.clone(), new.clone())),
            });
            let removed = before
                .keys()
                .filter(|key| !after.contains_key(*key))
                .map(|key| (key.clone(), Value::Null));

            Value::Object(changed.chain(removed).collect::<Map<_, _>>())
        }
        (_, to) => to.clone(),
    }
}

/// Set one JSON Pointer location, creating intermediate objects as needed.
#[must_use]
pub fn set_at(value: Value, path: &str, new_value: Value) -> Value {
    match pointer::segments(path) {
        Some(segments) => set_segments(value, &segments, new_value),
        None => new_value,
    }
}

fn set_segments(value: Value, segments: &[String], new_value: Value) -> Value {
    let Some((head, tail)) = segments.split_first() else {
        return new_value;
    };

    match value {
        Value::Array(mut items) => match head.parse::<usize>() {
            Ok(index) if index < items.len() => {
                let element = std::mem::replace(&mut items[index], Value::Null);
                items[index] = set_segments(element, tail, new_value);
                Value::Array(items)
            }
            _ => Value::Array(items),
        },
        Value::Object(mut fields) => {
            let child = fields.remove(head).unwrap_or(Value::Null);
            fields.insert(head.clone(), set_segments(child, tail, new_value));
            Value::Object(fields)
        }
        _ => Value::Object(Map::from_iter([(
            head.clone(),
            set_segments(Value::Null, tail, new_value),
        )])),
    }
}

/// Remove every `null` leaf. Keycloak representations are mostly null, and hiding those
/// fields is the difference between a readable screen and a wall of noise.
#[must_use]
pub fn prune_nulls(value: Value) -> Value {
    match value {
        Value::Object(fields) => Value::Object(
            fields
                .into_iter()
                .filter(|(_, value)| !value.is_null())
                .map(|(key, value)| (key, prune_nulls(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(prune_nulls).collect()),
        other => other,
    }
}
