//! Lossless, tolerant JSON handling.
//!
//! A representation fetched from Keycloak is never narrowed into a struct before being
//! written back: edits are applied as merge patches, so fields this build has never heard of
//! survive a round-trip untouched.

mod merge;
mod pointer;
mod splice;

pub use merge::{diff, merge, prune_nulls, set_at};
pub use pointer::{at, at_any, attribute, leaves, len_of};
pub use splice::splice;

use std::borrow::Cow;

use serde_json::Value;

/// Render any value as a single line fit for a table cell.
#[must_use]
pub fn to_line(value: &Value) -> Cow<'_, str> {
    match value {
        Value::Null => Cow::Borrowed(""),
        Value::String(text) => Cow::Borrowed(text.as_str()),
        Value::Bool(flag) => Cow::Borrowed(if *flag { "yes" } else { "no" }),
        Value::Number(number) => Cow::Owned(number.to_string()),
        Value::Array(items) => Cow::Owned(
            items
                .iter()
                .map(|item| to_line(item).into_owned())
                .collect::<Vec<_>>()
                .join(", "),
        ),
        Value::Object(_) => Cow::Owned(compact(value)),
    }
}

#[must_use]
pub fn compact(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unserialisable>".to_owned())
}

#[must_use]
pub fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "<unserialisable>".to_owned())
}

#[cfg(test)]
mod tests;
