use serde_json::Value;

/// Read a JSON Pointer (RFC 6901); any missing segment yields `None`.
#[must_use]
pub fn at<'v>(value: &'v Value, pointer: &str) -> Option<&'v Value> {
    if pointer.is_empty() {
        Some(value)
    } else {
        value.pointer(pointer)
    }
}

/// First pointer that resolves to a non-null value.
///
/// Keycloak renames fields far more often than it removes concepts, so projections list the
/// current name first and historical aliases after it.
#[must_use]
pub fn at_any<'v>(value: &'v Value, pointers: &[&str]) -> Option<&'v Value> {
    pointers
        .iter()
        .filter_map(|pointer| at(value, pointer))
        .find(|found| !found.is_null())
}

/// Look up a Keycloak attribute, unwrapping the `{"key": ["value"]}` shape.
#[must_use]
pub fn attribute<'v>(value: &'v Value, key: &str) -> Option<&'v Value> {
    value
        .get("attributes")
        .and_then(|attributes| attributes.get(key))
        .and_then(|found| match found {
            Value::Array(items) => items.first(),
            other => Some(other),
        })
}

/// Depth-first `(pointer, value)` pairs for every leaf. Empty objects and arrays are leaves.
#[must_use]
pub fn leaves(value: &Value) -> Vec<(String, &Value)> {
    fn walk<'v>(value: &'v Value, prefix: &str) -> Vec<(String, &'v Value)> {
        match value {
            Value::Object(fields) if !fields.is_empty() => fields
                .iter()
                .flat_map(|(key, child)| walk(child, &format!("{prefix}/{}", escape(key))))
                .collect(),
            Value::Array(items) if !items.is_empty() => items
                .iter()
                .enumerate()
                .flat_map(|(index, child)| walk(child, &format!("{prefix}/{index}")))
                .collect(),
            leaf => vec![(prefix.to_owned(), leaf)],
        }
    }

    walk(value, "")
}

#[must_use]
pub fn escape(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

#[must_use]
pub fn unescape(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}

/// Split a pointer into its decoded segments. `None` for a malformed pointer.
#[must_use]
pub fn segments(pointer: &str) -> Option<Vec<String>> {
    match pointer {
        "" => Some(Vec::new()),
        p if p.starts_with('/') => Some(p[1..].split('/').map(unescape).collect()),
        _ => None,
    }
}

/// Count of direct children, for "N roles" style columns.
#[must_use]
pub fn len_of(value: &Value) -> Option<usize> {
    match value {
        Value::Array(items) => Some(items.len()),
        Value::Object(fields) => Some(fields.len()),
        _ => None,
    }
}
