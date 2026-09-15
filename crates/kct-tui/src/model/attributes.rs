use std::sync::Arc;

use serde_json::{Map, Value};

use crate::text_input::TextInput;

/// Which half of a row is being worked on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Key,
    Value,
}

/// How the map stores its values.
///
/// Users, groups, roles and organizations use `{"k": ["v"]}`; realms and clients use
/// `{"k": "v"}`. Writing the wrong shape is accepted by the API and then ignored, which is
/// the worst kind of failure, so the shape is carried explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    ListValued,
    StringValued,
}

impl Shape {
    fn encode(self, value: &str) -> Value {
        match self {
            Self::ListValued => Value::Array(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|part| !part.is_empty())
                    .map(|part| Value::String(part.to_owned()))
                    .collect(),
            ),
            Self::StringValued => Value::String(value.to_owned()),
        }
    }

    fn decode(value: &Value) -> String {
        match value {
            Value::Array(items) => items
                .iter()
                .map(|item| kct_core::json::to_line(item).into_owned())
                .collect::<Vec<_>>()
                .join(", "),
            other => kct_core::json::to_line(other).into_owned(),
        }
    }
}

/// A key–value editor for an attribute map.
#[derive(Debug, Clone)]
pub struct Attributes {
    pub title: Arc<str>,
    pub pointer: &'static str,
    pub shape: Shape,
    pub rows: Arc<[(String, String)]>,
    pub cursor: usize,
    pub column: Column,
    pub editing: Option<TextInput>,
    /// Attribute names the realm's user profile declares.
    pub declared: Arc<[String]>,
    /// Whether this realm actually discards the ones it has not declared. When it does not,
    /// an undeclared key is perfectly normal and flagging it would be noise.
    pub discards_unmanaged: bool,
}

impl Attributes {
    #[must_use]
    pub fn open(
        title: impl Into<Arc<str>>,
        pointer: &'static str,
        shape: Shape,
        value: Option<&Value>,
        declared: Arc<[String]>,
        discards_unmanaged: bool,
    ) -> Self {
        let rows: Vec<(String, String)> = value
            .and_then(Value::as_object)
            .map(|map| {
                map.iter()
                    .map(|(key, value)| (key.clone(), Shape::decode(value)))
                    .collect()
            })
            .unwrap_or_default();

        Self {
            title: title.into(),
            pointer,
            shape,
            rows: Arc::from(rows),
            cursor: 0,
            column: Column::Value,
            editing: None,
            declared,
            discards_unmanaged,
        }
    }

    #[must_use]
    pub fn moved(self, delta: isize) -> Self {
        if self.rows.is_empty() {
            return self;
        }
        let last = self.rows.len() - 1;
        let cursor = self.cursor.saturating_add_signed(delta).min(last);
        Self { cursor, ..self }
    }

    #[must_use]
    pub fn switched(self) -> Self {
        let column = match self.column {
            Column::Key => Column::Value,
            Column::Value => Column::Key,
        };
        Self { column, ..self }
    }

    #[must_use]
    pub fn added(self) -> Self {
        let mut rows = self.rows.to_vec();
        rows.push((String::new(), String::new()));
        let cursor = rows.len() - 1;

        Self {
            rows: Arc::from(rows),
            cursor,
            column: Column::Key,
            editing: Some(TextInput::default()),
            ..self
        }
    }

    #[must_use]
    pub fn removed(self) -> Self {
        if self.rows.is_empty() {
            return self;
        }
        let rows: Vec<(String, String)> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != self.cursor)
            .map(|(_, row)| row.clone())
            .collect();
        let cursor = self.cursor.min(rows.len().saturating_sub(1));

        Self {
            rows: Arc::from(rows),
            cursor,
            ..self
        }
    }

    /// Replace the selected cell with what was typed.
    #[must_use]
    pub fn committed(self, text: &str) -> Self {
        let mut rows = self.rows.to_vec();
        if let Some(row) = rows.get_mut(self.cursor) {
            match self.column {
                Column::Key => text.trim().clone_into(&mut row.0),
                Column::Value => text.clone_into(&mut row.1),
            }
        }
        Self {
            rows: Arc::from(rows),
            editing: None,
            ..self
        }
    }

    /// The map as Keycloak wants it. Rows with a blank key are dropped rather than sent.
    #[must_use]
    pub fn to_json(&self) -> Value {
        Value::Object(
            self.rows
                .iter()
                .filter(|(key, _)| !key.trim().is_empty())
                .map(|(key, value)| (key.clone(), self.shape.encode(value)))
                .collect::<Map<_, _>>(),
        )
    }

    /// Keys this realm would silently discard on save. Empty when the realm allows unmanaged
    /// attributes, which is the common case once someone has turned it on.
    #[must_use]
    pub fn undeclared(&self) -> Vec<&str> {
        if !self.discards_unmanaged {
            return Vec::new();
        }
        self.rows
            .iter()
            .map(|(key, _)| key.as_str())
            .filter(|key| !key.trim().is_empty() && !self.declared.iter().any(|d| d == key))
            .collect()
    }
}
