use std::collections::BTreeSet;
use std::sync::Arc;

use serde_json::Value;

use super::commit::Commit;
use crate::text_input::TextInput;

/// A modal that fetches candidates, filters them and acts on the chosen ones.
#[derive(Debug, Clone)]
pub struct Picker {
    pub title: Arc<str>,
    pub input: TextInput,
    pub options: Arc<[Value]>,
    pub matches: Arc<[usize]>,
    pub cursor: usize,
    pub chosen: BTreeSet<usize>,
    /// Pointers tried in order to label a candidate.
    pub label: &'static [&'static str],
    pub commit: Commit,
    pub loading: bool,
    pub token: u64,
    /// Identifiers already attached, which the list must not offer again. Keycloak has an
    /// `/available` endpoint for role mappings but not for client scopes, so the second set
    /// is fetched and subtracted here.
    pub excluded: Arc<[String]>,
    /// Field the exclusion set is keyed on.
    pub exclude_key: &'static str,
    /// Pointer/value pairs a candidate must match. Keycloak silently ignores a client scope
    /// whose protocol differs from the client's, so offering one is worse than useless.
    pub require: Arc<[(&'static str, String)]>,
}

impl Picker {
    #[must_use]
    pub fn loading(
        title: impl Into<Arc<str>>,
        label: &'static [&'static str],
        commit: Commit,
        token: u64,
    ) -> Self {
        Self {
            title: title.into(),
            input: TextInput::default(),
            options: Arc::from(Vec::new()),
            matches: Arc::from(Vec::new()),
            cursor: 0,
            chosen: BTreeSet::new(),
            label,
            commit,
            loading: true,
            token,
            excluded: Arc::from(Vec::new()),
            exclude_key: "id",
            require: Arc::from(Vec::new()),
        }
    }

    #[must_use]
    pub fn requiring(self, require: Vec<(&'static str, String)>) -> Self {
        Self {
            require: Arc::from(require),
            ..self
        }
        .refiltered()
    }

    #[must_use]
    pub fn with_exclusions(self, excluded: Arc<[String]>) -> Self {
        Self { excluded, ..self }.refiltered()
    }

    #[must_use]
    pub fn with_options(self, options: Arc<[Value]>) -> Self {
        Self {
            options,
            loading: false,
            ..self
        }
        .refiltered()
    }

    #[must_use]
    pub fn refiltered(self) -> Self {
        let needle = self.input.value().trim().to_lowercase();
        let matches: Vec<usize> = self
            .options
            .iter()
            .enumerate()
            .filter(|(_, option)| !self.is_excluded(option) && self.is_eligible(option))
            .filter(|(_, option)| {
                needle.is_empty() || self.label_of(option).to_lowercase().contains(&needle)
            })
            .map(|(index, _)| index)
            .collect();

        let cursor = self.cursor.min(matches.len().saturating_sub(1));
        Self {
            matches: Arc::from(matches),
            cursor,
            ..self
        }
    }

    fn is_eligible(&self, option: &Value) -> bool {
        self.require.iter().all(|(pointer, expected)| {
            kct_core::json::at(option, pointer)
                .and_then(Value::as_str)
                .is_some_and(|actual| actual == expected)
        })
    }

    fn is_excluded(&self, option: &Value) -> bool {
        option
            .get(self.exclude_key)
            .and_then(Value::as_str)
            .is_some_and(|id| self.excluded.iter().any(|taken| taken == id))
    }

    #[must_use]
    pub fn label_of(&self, option: &Value) -> String {
        kct_core::json::at_any(option, self.label).map_or_else(
            || "(unnamed)".to_owned(),
            |found| kct_core::json::to_line(found).into_owned(),
        )
    }

    #[must_use]
    pub fn moved(self, delta: isize) -> Self {
        if self.matches.is_empty() {
            return self;
        }
        let last = self.matches.len() - 1;
        let cursor = self.cursor.saturating_add_signed(delta).min(last);
        Self { cursor, ..self }
    }

    /// Toggle the highlighted candidate. Multi-select is the point: granting four roles in
    /// one pass is the normal case, not the exception.
    #[must_use]
    pub fn toggled(self) -> Self {
        let Some(index) = self.matches.get(self.cursor).copied() else {
            return self;
        };
        let mut chosen = self.chosen;
        if !chosen.remove(&index) {
            chosen.insert(index);
        }
        Self { chosen, ..self }
    }

    /// The picked objects, in catalog order.
    #[must_use]
    pub fn picked(&self) -> Vec<Value> {
        let explicit: Vec<Value> = self
            .chosen
            .iter()
            .filter_map(|index| self.options.get(*index).cloned())
            .collect();

        // Confirming without ticking anything acts on what is highlighted.
        if explicit.is_empty() {
            self.matches
                .get(self.cursor)
                .and_then(|index| self.options.get(*index))
                .cloned()
                .into_iter()
                .collect()
        } else {
            explicit
        }
    }
}
