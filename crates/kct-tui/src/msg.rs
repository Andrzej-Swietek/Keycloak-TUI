//! Messages and effects — the two data types that carry everything in and out of [`update`].
//!
//! [`update`]: crate::update::update

use std::sync::Arc;

use crossterm::event::KeyEvent;
use kct_core::capability::ServerProfile;
use kct_core::{Error, Method};
use serde_json::Value;

/// Everything that can happen to the application.
#[derive(Debug, Clone)]
pub enum Msg {
    Key(KeyEvent),
    /// Periodic wake-up: expires toasts and refreshes the token countdown.
    Tick,
    Profile(Result<ServerProfile, Error>),
    Realms(Result<Arc<[String]>, Error>),
    /// The realm's declarative user profile, which decides whether attribute writes stick.
    UserProfile(Result<Value, Error>),
    /// The realm representation, read for its opt-in switches.
    RealmFlags(Result<Value, Error>),
    AuthFlows(Result<Arc<[Value]>, Error>),
    Mascots(crate::view::Scan),
    /// A page of list rows. The token lets a late reply from an abandoned screen be dropped
    /// instead of overwriting whatever the operator is looking at now.
    Rows {
        token: u64,
        result: Result<Page, Error>,
    },
    Item {
        token: u64,
        result: Result<Value, Error>,
    },
    Options {
        token: u64,
        result: Result<Arc<[Value]>, Error>,
    },
    /// Candidates the picker must not offer because they are already attached.
    Exclusions {
        token: u64,
        result: Result<Arc<[Value]>, Error>,
    },
    /// A write completed. `title` is what to say about it.
    Done {
        title: String,
        result: Result<Option<String>, Error>,
    },
    /// A read whose result is shown as text rather than parsed (adapter configs, exports).
    Text {
        title: String,
        result: Result<String, Error>,
    },
    Notice(Notice),
    /// One completed API call, for the request log.
    Logged {
        method: Method,
        path: String,
        outcome: Result<u16, String>,
    },
    /// The terminal changed size; `rows` is what the listing can now show.
    Viewport(usize),
}

/// What a fetch is for, so its reply knows where to land.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fetch {
    Rows,
    Item,
    /// Candidates for a picker modal.
    Options,
    /// Candidates a picker must exclude.
    Exclusions,
}

/// A transient message shown at the bottom of the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub text: String,
    pub level: Level,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Success,
    Warning,
    Failure,
}

impl Notice {
    #[must_use]
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: Level::Info,
        }
    }

    #[must_use]
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: Level::Success,
        }
    }

    #[must_use]
    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: Level::Warning,
        }
    }

    #[must_use]
    pub fn failure(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: Level::Failure,
        }
    }

    /// Render an error as a notice, keeping its remediation hint.
    #[must_use]
    pub fn from_error(context: &str, error: &Error) -> Self {
        Self::failure(match error.hint() {
            Some(hint) => format!("{context}: {error}\n{hint}"),
            None => format!("{context}: {error}"),
        })
    }
}

/// One page of rows, with the document they were read out of.
///
/// Most collections are their own endpoint and the envelope is the array itself. A few live
/// inside a larger document, and writing one of those rows means putting that whole document
/// back — so it has to survive the trip.
#[derive(Debug, Clone)]
pub struct Page {
    pub rows: Arc<[Value]>,
    pub envelope: Arc<Value>,
}
