//! Query parameters for Keycloak's list endpoints.
//!
//! Built functionally: every `with_*` consumes and returns `Self`, so a query can be
//! threaded through a pipeline without a single mutable borrow.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// How many rows a page holds by default. Keycloak's own console uses 10–20; a terminal has
/// more vertical room, and over-fetching a little keeps scrolling smooth.
pub const DEFAULT_PAGE_SIZE: u32 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub first: u32,
    pub max: u32,
    /// Free-text search, matched server-side against the resource's natural fields.
    pub search: Option<String>,
    /// Require an exact match rather than the default "contains" semantics.
    pub exact: bool,
    /// Ask for the trimmed representation: Keycloak omits attributes and other heavy fields.
    ///
    /// A row fetched this way is incomplete and must not be written back. Open it through the
    /// resource's item endpoint first.
    pub brief: bool,
    /// Attribute query (`q=key1:value1 key2:value2`), supported by users and groups.
    pub attributes: BTreeMap<String, String>,
    /// Endpoint-specific parameters we do not model explicitly. The escape hatch that lets
    /// a new Keycloak filter be used without a code change.
    pub extra: BTreeMap<String, String>,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            first: 0,
            max: DEFAULT_PAGE_SIZE,
            search: None,
            exact: false,
            brief: false,
            attributes: BTreeMap::new(),
            extra: BTreeMap::new(),
        }
    }
}

impl Query {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A query for a single page starting at row `first`.
    #[must_use]
    pub fn page(first: u32, max: u32) -> Self {
        Self {
            first,
            max,
            ..Self::default()
        }
    }

    /// Fetch everything the endpoint will give us in one go. Reserved for genuinely small
    /// collections (realms, authentication flows, client scopes) — never for users.
    #[must_use]
    pub fn unbounded() -> Self {
        Self {
            max: 20_000,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn with_search(self, search: impl Into<String>) -> Self {
        let search = search.into();
        Self {
            search: (!search.trim().is_empty()).then_some(search),
            ..self
        }
    }

    #[must_use]
    pub fn with_exact(self, exact: bool) -> Self {
        Self { exact, ..self }
    }

    #[must_use]
    pub fn with_brief(self, brief: bool) -> Self {
        Self { brief, ..self }
    }

    #[must_use]
    pub fn with_attribute(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let mut attributes = self.attributes;
        attributes.insert(key.into(), value.into());
        Self { attributes, ..self }
    }

    #[must_use]
    pub fn with_param(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let mut extra = self.extra;
        extra.insert(key.into(), value.into());
        Self { extra, ..self }
    }

    /// Advance to the next page, keeping every filter intact.
    #[must_use]
    pub fn next_page(self) -> Self {
        Self {
            first: self.first.saturating_add(self.max),
            ..self
        }
    }

    #[must_use]
    pub fn previous_page(self) -> Self {
        Self {
            first: self.first.saturating_sub(self.max),
            ..self
        }
    }

    #[must_use]
    pub fn page_index(&self) -> u32 {
        self.first.checked_div(self.max).unwrap_or(0)
    }

    /// Render as `key=value` pairs. Booleans are only emitted when true — Keycloak treats an
    /// explicit `exact=false` the same as absent, and a shorter URL is easier to eyeball in
    /// the request log.
    #[must_use]
    pub fn to_pairs(&self) -> Vec<(String, String)> {
        let base = [
            ("first".to_owned(), self.first.to_string()),
            ("max".to_owned(), self.max.to_string()),
        ];

        let search = self.search.iter().map(|s| ("search".to_owned(), s.clone()));

        let flags = [("exact", self.exact), ("briefRepresentation", self.brief)]
            .into_iter()
            .filter(|(_, on)| *on)
            .map(|(name, _)| (name.to_owned(), "true".to_owned()));

        let attribute_query = (!self.attributes.is_empty()).then(|| {
            let joined = self
                .attributes
                .iter()
                .fold(String::new(), |mut acc, (k, v)| {
                    if !acc.is_empty() {
                        acc.push(' ');
                    }
                    let _ = write!(acc, "{k}:{v}");
                    acc
                });
            ("q".to_owned(), joined)
        });

        base.into_iter()
            .chain(search)
            .chain(flags)
            .chain(attribute_query)
            .chain(self.extra.clone())
            .collect()
    }

    /// Human-readable summary for the status bar ("page 3 · search 'ala' · exact").
    #[must_use]
    pub fn describe(&self) -> String {
        let mut parts = vec![format!("page {}", self.page_index() + 1)];
        if let Some(search) = &self.search {
            parts.push(format!("search {search:?}"));
        }
        if self.exact {
            parts.push("exact".to_owned());
        }
        parts.extend(self.attributes.iter().map(|(k, v)| format!("{k}={v}")));
        parts.join(" · ")
    }
}

#[cfg(test)]
mod tests;
