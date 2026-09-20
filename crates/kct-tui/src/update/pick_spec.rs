//! Where a picker gets its candidates, and what confirming does with them.

use kct_core::ApiPath;

use crate::model::Commit;

/// Where a picker gets its candidates, and what confirming does with them.
///
/// Build with [`Spec::from`] and add only what the case needs:
///
/// ```ignore
/// Spec::from(title, source, &["/name"], commit).without(vec![own_id])
/// ```
pub(crate) struct Spec {
    pub title: String,
    /// Candidates already in hand. When set, the picker opens without a request — a step's
    /// requirement choices travel with the step itself.
    pub options: Option<Vec<serde_json::Value>>,
    /// Candidate fields that must match for the server to accept the pairing.
    pub require: Vec<(&'static str, String)>,
    pub source: ApiPath,
    /// Fetched separately and subtracted from the candidates.
    pub exclude: Vec<ApiPath>,
    /// Ids to drop without a request.
    pub without: Vec<String>,
    pub label: &'static [&'static str],
    pub commit: Commit,
}

impl Spec {
    /// A picker over everything `source` returns.
    pub(crate) fn from(
        title: impl Into<String>,
        source: ApiPath,
        label: &'static [&'static str],
        commit: Commit,
    ) -> Self {
        Self {
            title: title.into(),
            options: None,
            require: Vec::new(),
            exclude: Vec::new(),
            without: Vec::new(),
            source,
            label,
            commit,
        }
    }

    /// Offer these instead of fetching.
    #[must_use]
    pub(crate) fn of(self, options: Vec<serde_json::Value>) -> Self {
        Self {
            options: Some(options),
            ..self
        }
    }

    /// Only candidates whose `pointer` equals `value`.
    #[must_use]
    pub(crate) fn requiring(self, pointer: &'static str, value: String) -> Self {
        let mut require = self.require;
        require.push((pointer, value));
        Self { require, ..self }
    }

    /// Subtract whatever these paths return.
    #[must_use]
    pub(crate) fn excluding(self, exclude: Vec<ApiPath>) -> Self {
        Self { exclude, ..self }
    }

    /// Drop these ids without a request.
    #[must_use]
    pub(crate) fn without(self, without: Vec<String>) -> Self {
        Self { without, ..self }
    }
}
