use serde_json::Value;

use crate::json;

/// The JSON shape a collection endpoint actually returns.
///
/// Keycloak is not uniform: most collections are arrays, `/keys` wraps them in an object,
/// `/serverinfo` nests several catalogs, and a few endpoints are singletons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Array,
    /// An object with the rows at this pointer (`/keys`, `/features`).
    ArrayAt(&'static str),
    /// An object whose entries are the rows; each becomes `{"name": key, ...value}`.
    EntriesAt(&'static str),
    /// A single object that is itself the resource.
    Singleton,
}

impl Shape {
    /// Normalise a response body into rows. A body that does not match this shape yields no
    /// rows.
    #[must_use]
    pub fn rows(self, body: Value) -> Vec<Value> {
        match self {
            Self::Array => match body {
                Value::Array(items) => items,
                other => vec![other],
            },
            Self::ArrayAt(pointer) => json::at(&body, pointer)
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            Self::EntriesAt(pointer) => json::at(&body, pointer)
                .and_then(Value::as_object)
                .map(|entries| {
                    entries
                        .iter()
                        .map(|(name, value)| {
                            json::merge(value.clone(), serde_json::json!({ "name": name }))
                        })
                        .collect()
                })
                .unwrap_or_default(),
            Self::Singleton => vec![body],
        }
    }
}
