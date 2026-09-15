//! A fetched page of results.
//!
//! Items are held behind an [`Arc`] so cloning application state — which the update loop
//! does on every keystroke — stays O(1) regardless of how many rows are loaded.

use std::sync::Arc;

use crate::query::Query;

#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Arc<[T]>,
    /// The query that produced this page, so "next page" needs no extra bookkeeping.
    pub query: Query,
    /// Total across all pages. `None` where the endpoint has no `/count` companion.
    pub total: Option<u64>,
}

impl<T> Page<T> {
    #[must_use]
    pub fn new(items: impl Into<Arc<[T]>>, query: Query) -> Self {
        Self {
            items: items.into(),
            query,
            total: None,
        }
    }

    #[must_use]
    pub fn with_total(self, total: Option<u64>) -> Self {
        Self { total, ..self }
    }

    #[must_use]
    pub fn empty(query: Query) -> Self {
        Self {
            items: Arc::from(Vec::new()),
            query,
            total: None,
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// A full page strongly suggests more rows exist; Keycloak list endpoints give no
    /// explicit "has more" signal, so this is the best available heuristic.
    #[must_use]
    pub fn probably_has_more(&self) -> bool {
        match self.total {
            Some(total) => (u64::from(self.query.first) + self.len() as u64) < total,
            None => u32::try_from(self.len()).unwrap_or(u32::MAX) >= self.query.max,
        }
    }

    /// Row range of this page in the overall collection, 1-based, for the status bar.
    #[must_use]
    pub fn range(&self) -> (u64, u64) {
        let start = u64::from(self.query.first) + 1;
        (start, u64::from(self.query.first) + self.len() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_page_implies_more_rows_when_no_total_is_known() {
        let full = Page::new(vec![0_u8; 100], Query::page(0, 100));
        assert!(full.probably_has_more());

        let partial = Page::new(vec![0_u8; 7], Query::page(0, 100));
        assert!(!partial.probably_has_more());
    }

    #[test]
    fn a_known_total_beats_the_heuristic() {
        let page = Page::new(vec![0_u8; 100], Query::page(0, 100)).with_total(Some(100));
        assert!(!page.probably_has_more());
        assert_eq!(page.range(), (1, 100));
    }
}
