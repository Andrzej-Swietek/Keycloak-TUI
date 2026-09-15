use std::sync::Arc;

use kct_core::catalog::Danger;

use crate::effect::Effect;
use crate::spotlight::Entry;
use crate::text_input::TextInput;

/// A modal on top of everything else.
impl Overlay {
    /// The picker is much larger than the other variants and the model is cloned on every
    /// keystroke, so it travels behind a pointer.
    #[must_use]
    pub fn picker(picker: super::Picker) -> Self {
        Self::Picker(Box::new(picker))
    }
}

#[derive(Debug, Clone)]
pub enum Overlay {
    Spotlight(Spotlight),
    Picker(Box<super::Picker>),
    Attributes(super::Attributes),
    Help {
        scroll: u16,
    },
    Confirm(Confirm),
    Prompt(Prompt),
    /// A scrollable read-only body: an export, an adapter config, a decoded token.
    Viewer {
        title: Arc<str>,
        body: Arc<str>,
        scroll: u16,
    },
    RequestLog {
        cursor: usize,
    },
}

#[derive(Debug, Clone)]
pub struct Spotlight {
    pub input: TextInput,
    /// Set when the typed text is a vim-style command rather than a search.
    pub ex: Option<crate::spotlight::Ex>,
    pub entries: Arc<[Entry]>,
    pub matches: Arc<[usize]>,
    pub cursor: usize,
}

impl Spotlight {
    #[must_use]
    pub fn new(entries: Vec<Entry>) -> Self {
        let matches: Vec<usize> = (0..entries.len()).collect();
        Self {
            input: TextInput::default(),
            ex: None,
            entries: Arc::from(entries),
            matches: Arc::from(matches),
            cursor: 0,
        }
    }

    /// Re-rank against a specific query. Used by tests and by "open the palette already
    /// filtered".
    #[must_use]
    pub fn researched_with(self, needle: &str) -> Self {
        Self {
            input: TextInput::new(needle),
            ..self
        }
        .researched()
    }

    #[must_use]
    pub fn researched(self) -> Self {
        let matches = crate::spotlight::search(&self.entries, self.input.value());
        Self {
            ex: crate::spotlight::Ex::parse(self.input.value()),
            matches: Arc::from(matches),
            cursor: 0,
            ..self
        }
    }

    #[must_use]
    pub fn selected(&self) -> Option<&Entry> {
        self.matches
            .get(self.cursor)
            .and_then(|index| self.entries.get(*index))
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
}

#[derive(Debug, Clone)]
pub struct Confirm {
    pub title: Arc<str>,
    pub body: Arc<str>,
    pub danger: Danger,
    /// For [`Danger::Destructive`], the exact text that must be typed to proceed. Making the
    /// operator retype an object's name is the cheapest available guard against deleting the
    /// wrong realm at 3am.
    pub expect: Option<Arc<str>>,
    pub typed: TextInput,
    pub on_confirm: Arc<Vec<Effect>>,
}

impl Confirm {
    #[must_use]
    pub fn is_satisfied(&self) -> bool {
        match &self.expect {
            None => true,
            Some(expected) => self.typed.value() == &**expected,
        }
    }
}

/// What a prompt's answer will be used for.
#[derive(Debug, Clone)]
pub enum Ask {
    /// Create an object whose only required field is a name.
    CreateNamed {
        resource: &'static str,
        field: &'static str,
    },
    /// Server-side search on the current list.
    Search,
    /// Set a user's password.
    SetPassword { temporary: bool },
    /// A whole representation, pasted, to create as-is.
    CreateJson { resource: &'static str },
    /// Add a sub-flow of this name to the flow whose executions are open.
    AddSubFlow,
    /// Write the viewer's contents to this path.
    SaveToFile { body: Arc<str> },
    /// Read a realm representation from this path and partially import it.
    ImportFrom,
}

#[derive(Debug, Clone)]
pub struct Prompt {
    pub title: Arc<str>,
    pub help: Arc<str>,
    pub input: TextInput,
    pub secret: bool,
    pub ask: Ask,
}
