use crate::capability::Requirement;

/// How dangerous an action is, which decides what confirmation it gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Danger {
    /// Read-only or trivially reversible.
    Safe,
    /// Writes; a single keypress confirmation.
    Confirm,
    /// Irreversible or wide blast radius; confirmed by typing the object's name.
    Destructive,
}

/// What an action does. The engine knows how to perform each variant generically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Create,
    Edit,
    Duplicate,
    Delete,
    /// `POST` to a sub-path with no body, e.g. `logout`.
    Post(&'static str),
    /// `DELETE` a sub-path, e.g. a client's rotated secret.
    Remove(&'static str),
    /// `GET` a sub-path and show the result.
    Fetch(&'static str),
    Export,
    Import,
    /// Handled by bespoke UI code, keyed by name.
    Special(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub struct Action {
    pub id: &'static str,
    pub title: &'static str,
    pub verb: Verb,
    pub danger: Danger,
    pub requirement: Requirement,
    /// Extra words the command palette should match on.
    pub keywords: &'static [&'static str],
}

impl Action {
    #[must_use]
    pub const fn new(id: &'static str, title: &'static str, verb: Verb, danger: Danger) -> Self {
        Self {
            id,
            title,
            verb,
            danger,
            requirement: Requirement::Always,
            keywords: &[],
        }
    }

    #[must_use]
    pub const fn needs(self, requirement: Requirement) -> Self {
        Self {
            requirement,
            ..self
        }
    }

    #[must_use]
    pub const fn keywords(self, keywords: &'static [&'static str]) -> Self {
        Self { keywords, ..self }
    }
}
