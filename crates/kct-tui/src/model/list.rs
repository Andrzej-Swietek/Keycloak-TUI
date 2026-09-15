use std::sync::Arc;

use kct_core::catalog::Resource;
use kct_core::{Error, Query};
use serde_json::Value;

use crate::text_input::TextInput;

/// State of the list pane.
#[derive(Debug, Clone, Default)]
pub struct List {
    pub rows: Arc<[Value]>,
    /// The document `rows` were read out of, kept whole for resources whose items live inside
    /// it and can only be written back as part of it.
    pub envelope: Arc<Value>,
    /// Indices of `rows` surviving the local filter, in display order.
    pub visible: Arc<[usize]>,
    pub query: Query,
    pub cursor: usize,
    pub scroll: usize,
    pub filter: TextInput,
    pub filtering: bool,
    pub loading: bool,
    pub error: Option<Error>,
    pub total: Option<u64>,
    /// Identifies the in-flight request; replies carrying a different token are stale.
    pub token: u64,
}

impl List {
    #[must_use]
    pub fn loading(token: u64, query: Query) -> Self {
        Self {
            loading: true,
            token,
            query,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn selected(&self) -> Option<&Value> {
        self.visible
            .get(self.cursor)
            .and_then(|index| self.rows.get(*index))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.visible.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.visible.is_empty()
    }

    /// Recompute the visible set from the current filter text.
    ///
    /// Matching is over the *rendered* row, not the raw JSON, so what the operator sees is
    /// what they can search — including computed columns like a client's type.
    #[must_use]
    pub fn refiltered(self, resource: &Resource) -> Self {
        let needle = self.filter.value().trim().to_lowercase();

        let visible: Vec<usize> = if needle.is_empty() {
            (0..self.rows.len()).collect()
        } else {
            self.rows
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    resource
                        .columns
                        .iter()
                        .any(|column| column.value.render(row).to_lowercase().contains(&needle))
                        || resource.label_of(row).to_lowercase().contains(&needle)
                })
                .map(|(index, _)| index)
                .collect()
        };

        let cursor = self.cursor.min(visible.len().saturating_sub(1));
        Self {
            visible: Arc::from(visible),
            cursor,
            ..self
        }
    }

    #[must_use]
    pub fn moved(self, delta: isize) -> Self {
        if self.visible.is_empty() {
            return self;
        }
        let last = self.visible.len() - 1;
        let cursor = self.cursor.saturating_add_signed(delta).min(last);
        Self { cursor, ..self }
    }

    #[must_use]
    pub fn moved_to(self, cursor: usize) -> Self {
        let cursor = cursor.min(self.visible.len().saturating_sub(1));
        Self { cursor, ..self }
    }

    /// Keep the cursor inside the window of `height` rows.
    #[must_use]
    pub fn scrolled_into_view(self, height: usize) -> Self {
        let height = height.max(1);
        let scroll = if self.cursor < self.scroll {
            self.cursor
        } else if self.cursor >= self.scroll + height {
            self.cursor - height + 1
        } else {
            self.scroll
        };
        Self { scroll, ..self }
    }
}
