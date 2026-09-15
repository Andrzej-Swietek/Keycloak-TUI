use serde_json::Value;

use crate::capability::Requirement;
use crate::json;

use super::encoding::{Encoding, decode_text, encode_text, format_seconds};

/// How one field of a representation is edited.
#[derive(Debug, Clone, Copy)]
pub enum FieldKind {
    Text,
    Multiline,
    Bool,
    Integer,
    /// Seconds, displayed as a human duration.
    Seconds,
    /// Rendered as a mask of the same length; the value is not shown.
    Secret,
    Choice(&'static [&'static str]),
    /// A list of strings: redirect URIs, web origins.
    List,
    /// Keycloak's `{"key": ["value"]}` map: users, groups, roles, organizations, and every
    /// component's `config`.
    Attributes,
    /// A `{"key": "value"}` map: realm and client attributes, protocol-mapper and identity-
    /// provider config. Writing the [`FieldKind::Attributes`] shape into one is accepted and
    /// has no effect.
    StringMap,
    /// Shown but not editable.
    ReadOnly,
    /// Arbitrary nested JSON.
    Json,
}

#[derive(Debug, Clone, Copy)]
pub struct Field {
    pub label: &'static str,
    pub pointer: &'static str,
    pub kind: FieldKind,
    pub encoding: Encoding,
    pub help: &'static str,
    pub requirement: Requirement,
}

impl Field {
    #[must_use]
    pub const fn new(label: &'static str, pointer: &'static str, kind: FieldKind) -> Self {
        Self {
            label,
            pointer,
            kind,
            encoding: Encoding::Native,
            help: "",
            requirement: Requirement::Always,
        }
    }

    /// A field stored as a string inside an `attributes` map.
    #[must_use]
    pub const fn stringly(label: &'static str, pointer: &'static str, kind: FieldKind) -> Self {
        Self {
            encoding: Encoding::Stringly,
            ..Self::new(label, pointer, kind)
        }
    }

    /// A field stored as `["value"]` inside a component `config` map.
    #[must_use]
    pub const fn config(label: &'static str, pointer: &'static str, kind: FieldKind) -> Self {
        Self {
            encoding: Encoding::ConfigList,
            ..Self::new(label, pointer, kind)
        }
    }

    #[must_use]
    pub const fn help(self, help: &'static str) -> Self {
        Self { help, ..self }
    }

    #[must_use]
    pub const fn needs(self, requirement: Requirement) -> Self {
        Self {
            requirement,
            ..self
        }
    }

    /// The decoded, native-JSON value of this field.
    #[must_use]
    pub fn read(&self, object: &Value) -> Option<Value> {
        let stored = json::at(object, self.pointer)?;
        let unwrapped = match self.encoding {
            Encoding::ConfigList => match stored {
                Value::Array(items) => items.first()?,
                other => other,
            },
            _ => stored,
        };

        match (self.encoding, unwrapped) {
            (Encoding::Native, value) => Some(value.clone()),
            (_, Value::String(text)) => Some(decode_text(text, self.kind)),
            (_, value) => Some(value.clone()),
        }
    }

    /// Set this field from a native value, re-encoding it the way the server expects.
    #[must_use]
    pub fn write(&self, object: Value, value: Value) -> Value {
        let encoded = match self.encoding {
            Encoding::Native => value,
            Encoding::Stringly => encode_text(&value),
            Encoding::ConfigList => Value::Array(vec![encode_text(&value)]),
        };
        json::set_at(object, self.pointer, encoded)
    }

    /// One-line rendering for read-only display.
    #[must_use]
    pub fn display(&self, object: &Value) -> String {
        self.read(object)
            .map(|value| match (self.kind, &value) {
                (FieldKind::Secret, Value::String(text)) if !text.is_empty() => {
                    "•".repeat(text.chars().count().min(24))
                }
                (FieldKind::Seconds, Value::Number(number)) => number
                    .as_i64()
                    .map_or_else(|| number.to_string(), format_seconds),
                _ => json::to_line(&value).into_owned(),
            })
            .unwrap_or_default()
    }
}
