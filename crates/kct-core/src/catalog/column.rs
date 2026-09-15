use serde_json::Value;

use crate::json;

/// How a cell is derived from a row. Every variant is a pure projection of JSON.
#[derive(Debug, Clone, Copy)]
pub enum Projection {
    At(&'static str),
    /// First pointer that resolves: current name first, historical aliases after, which is
    /// how a renamed field stays rendered across versions.
    AtAny(&'static [&'static str]),
    Attribute(&'static str),
    Count(&'static str),
    Bool {
        pointer: &'static str,
        yes: &'static str,
        no: &'static str,
    },
    EpochMillis(&'static str),
    Join {
        pointer: &'static str,
        separator: &'static str,
    },
    /// Anything the declarative forms cannot express.
    Compute(fn(&Value) -> String),
}

impl Projection {
    /// Render against a row. Never fails: a missing value is an empty cell.
    #[must_use]
    pub fn render(&self, row: &Value) -> String {
        match *self {
            Self::At(pointer) => rendered(json::at(row, pointer)),
            Self::AtAny(pointers) => rendered(json::at_any(row, pointers)),
            Self::Attribute(key) => rendered(json::attribute(row, key)),
            Self::Count(pointer) => json::at(row, pointer)
                .and_then(json::len_of)
                .map(|count| count.to_string())
                .unwrap_or_default(),
            Self::Bool { pointer, yes, no } => {
                match json::at(row, pointer).and_then(Value::as_bool) {
                    Some(true) => yes.to_owned(),
                    Some(false) => no.to_owned(),
                    None => String::new(),
                }
            }
            Self::EpochMillis(pointer) => json::at(row, pointer)
                .and_then(Value::as_i64)
                .map(format_epoch_millis)
                .unwrap_or_default(),
            Self::Join { pointer, separator } => json::at(row, pointer)
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .map(|item| json::to_line(item).into_owned())
                        .collect::<Vec<_>>()
                        .join(separator)
                })
                .unwrap_or_default(),
            Self::Compute(project) => project(row),
        }
    }
}

fn rendered(value: Option<&Value>) -> String {
    value
        .map(|found| json::to_line(found).into_owned())
        .unwrap_or_default()
}

fn format_epoch_millis(millis: i64) -> String {
    chrono::DateTime::from_timestamp_millis(millis).map_or_else(
        || millis.to_string(),
        |instant| {
            instant
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        },
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Width {
    Fixed(u16),
    /// Share of the leftover space, proportional to the weight.
    Flex(u16),
}

/// Semantic colouring; the theme decides what each one looks like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Primary,
    Normal,
    Muted,
    Good,
    Warn,
    Bad,
    /// Green when truthy, red when falsy.
    Flag,
}

#[derive(Debug, Clone, Copy)]
pub struct Column {
    pub header: &'static str,
    pub value: Projection,
    pub width: Width,
    pub tone: Tone,
}

impl Column {
    #[must_use]
    pub const fn new(header: &'static str, value: Projection, width: Width) -> Self {
        Self {
            header,
            value,
            width,
            tone: Tone::Normal,
        }
    }

    #[must_use]
    pub const fn toned(self, tone: Tone) -> Self {
        Self { tone, ..self }
    }
}
