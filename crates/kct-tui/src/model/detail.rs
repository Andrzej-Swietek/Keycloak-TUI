use std::sync::Arc;

use kct_core::Error;
use serde_json::Value;

use crate::text_input::TextInput;

/// What a successful save leaves on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AfterSave {
    /// Reopen the object, showing what the server stored.
    #[default]
    Reopen,
    /// Return to the listing.
    Close,
}

/// An in-progress edit of one form field.
#[derive(Debug, Clone)]
pub struct FieldEdit {
    pub field: usize,
    pub input: TextInput,
    /// Values this field is known to accept, in declaration order.
    pub suggestions: Arc<[String]>,
    /// Indices of `suggestions` matching what has been typed.
    pub matches: Arc<[usize]>,
    /// Which suggestion is highlighted, if any. `None` means "keep what I typed".
    pub highlight: Option<usize>,
}

impl FieldEdit {
    #[must_use]
    pub fn new(field: usize, input: TextInput, suggestions: Vec<String>) -> Self {
        let matches: Vec<usize> = (0..suggestions.len()).collect();
        Self {
            field,
            input,
            suggestions: Arc::from(suggestions),
            matches: Arc::from(matches),
            highlight: None,
        }
    }

    /// Re-narrow the suggestions after a keystroke.
    #[must_use]
    pub fn refiltered(self) -> Self {
        let matches = crate::update::suggest_matching(&self.suggestions, self.input.value());
        Self {
            matches: Arc::from(matches),
            highlight: None,
            ..self
        }
    }

    #[must_use]
    pub fn moved(self, delta: isize) -> Self {
        if self.matches.is_empty() {
            return self;
        }
        let last = self.matches.len() - 1;
        let highlight = match self.highlight {
            None if delta > 0 => 0,
            None => last,
            Some(current) => current.saturating_add_signed(delta).min(last),
        };
        Self {
            highlight: Some(highlight),
            ..self
        }
    }

    /// What committing should write: the highlighted suggestion, else the typed text.
    #[must_use]
    pub fn value(&self) -> String {
        self.highlight
            .and_then(|index| self.matches.get(index))
            .and_then(|index| self.suggestions.get(*index))
            .cloned()
            .unwrap_or_else(|| self.input.value().to_owned())
    }
}

/// State of the detail pane.
#[derive(Debug, Clone)]
pub struct Detail {
    pub id: String,
    pub title: Arc<str>,
    /// The representation exactly as fetched — the base every save merges onto.
    pub original: Value,
    /// The same, with the operator's edits applied.
    pub draft: Value,
    pub tab: usize,
    pub cursor: usize,
    pub scroll: u16,
    pub editing: Option<FieldEdit>,
    pub after_save: AfterSave,
    pub loading: bool,
    pub error: Option<Error>,
    pub token: u64,
}

impl Detail {
    #[must_use]
    pub fn loading(id: impl Into<String>, title: impl Into<Arc<str>>, token: u64) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            original: Value::Null,
            draft: Value::Null,
            tab: 0,
            cursor: 0,
            scroll: 0,
            editing: None,
            after_save: AfterSave::default(),
            loading: true,
            error: None,
            token,
        }
    }

    /// A detail over an object already in hand, with no fetch pending.
    ///
    /// `original` and `draft` both start as `object`, so [`Detail::is_dirty`] is false.
    #[must_use]
    pub fn loaded(id: impl Into<String>, title: impl Into<Arc<str>>, object: Value) -> Self {
        Self {
            original: object.clone(),
            draft: object,
            loading: false,
            ..Self::loading(id, title, 0)
        }
    }

    /// The pending change set: what a save would actually send.
    #[must_use]
    pub fn pending_changes(&self) -> Value {
        kct_core::json::diff(&self.original, &self.draft)
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.original != self.draft
    }
}
