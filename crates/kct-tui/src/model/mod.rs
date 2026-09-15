//! The entire application state, as one value.
//!
//! [`Model`] is threaded by value through [`crate::update::update`] and rebuilt each step;
//! collections sit behind `Arc`, so a step copies pointers and not a realm's worth of JSON.

mod attributes;
mod commit;
mod detail;
mod journal;
mod list;
mod nav;
mod overlay;
mod picker;
mod prefs;
mod sidebar;
mod session;

pub use attributes::{Attributes, Column as AttributeColumn, Shape as AttributeShape};
pub use commit::Commit;
pub use detail::{AfterSave, Detail, FieldEdit};
pub use journal::{LogLine, Toast};
pub use list::List;
pub use nav::{Location, Nav};
pub use overlay::{Ask, Confirm, Overlay, Prompt, Spotlight};
pub use picker::Picker;
pub use prefs::{Mascot, Prefs};
pub use sidebar::{SidebarRow, sidebar_rows};
pub use session::{Session, UnmanagedAttributes};

use std::collections::VecDeque;
use std::time::Instant;

use kct_core::Context;
use kct_core::catalog::Resource;

use crate::msg::{Level, Notice};
use crate::theme::Theme;

/// How long a notice stays on screen.
pub const NOTICE_TTL: std::time::Duration = std::time::Duration::from_secs(6);

/// How many API calls the request log keeps.
const LOG_CAPACITY: usize = 200;

#[derive(Debug, Clone)]
pub struct Model {
    pub session: Session,
    pub nav: Nav,
    pub list: List,
    pub detail: Option<Detail>,
    pub overlay: Option<Overlay>,
    pub toasts: Vec<Toast>,
    pub log: VecDeque<LogLine>,
    pub inflight: u32,
    pub next_token: u64,
    pub quit: bool,
    pub theme: Theme,
    /// Rows the list pane can show; the view reports it so paging can size itself.
    pub viewport_rows: usize,
    /// Advances on every tick; drives the mascot's animation and nothing else.
    pub ticks: u64,
    pub prefs: Prefs,
    /// Art the operator dropped into the mascot directory, rescanned on request.
    pub mascots: crate::view::Scan,
}

impl Model {
    #[must_use]
    pub fn new(session: Session, resource: &'static Resource) -> Self {
        let context = Context::realm(&*session.realm);
        Self {
            nav: Nav::rooted(resource, context),
            session,
            list: List::default(),
            detail: None,
            overlay: None,
            toasts: Vec::new(),
            mascots: crate::view::Scan::default(),
            log: VecDeque::with_capacity(LOG_CAPACITY),
            inflight: 0,
            next_token: 1,
            quit: false,
            theme: Theme::default(),
            viewport_rows: 20,
            ticks: 0,
            prefs: Prefs::default(),
        }
    }

    #[must_use]
    pub fn resource(&self) -> &'static Resource {
        self.nav.current().resource
    }

    #[must_use]
    pub fn context(&self) -> &Context {
        &self.nav.current().context
    }

    /// Hand out a request token, so a reply can be matched to the screen that asked for it.
    #[must_use]
    pub fn issue_token(self) -> (Self, u64) {
        let token = self.next_token;
        (
            Self {
                next_token: token + 1,
                ..self
            },
            token,
        )
    }

    /// Mark the open detail to close once its save succeeds. A no-op with nothing open.
    #[must_use]
    pub fn closing_after_save(self) -> Self {
        match self.detail {
            Some(detail) => Self {
                detail: Some(Detail {
                    after_save: AfterSave::Close,
                    ..detail
                }),
                ..self
            },
            None => self,
        }
    }

    #[must_use]
    pub fn noticed(self, notice: Notice) -> Self {
        let mut toasts = self.toasts;
        toasts.push(Toast {
            notice,
            at: Instant::now(),
        });
        // Only the newest few are legible at once.
        let toasts = toasts.into_iter().rev().take(3).rev().collect();
        Self { toasts, ..self }
    }

    #[must_use]
    pub fn expire_toasts(self) -> Self {
        let now = Instant::now();
        let toasts = self
            .toasts
            .into_iter()
            .filter(|toast| {
                // Failures stay until something else replaces them: an operator who looked
                // away must still find out that a write did not happen.
                toast.notice.level == Level::Failure || now.duration_since(toast.at) < NOTICE_TTL
            })
            .collect();
        Self { toasts, ..self }
    }

    #[must_use]
    pub fn logged(self, line: LogLine) -> Self {
        let mut log = self.log;
        if log.len() == LOG_CAPACITY {
            log.pop_front();
        }
        log.push_back(line);
        Self { log, ..self }
    }

    #[must_use]
    pub fn with_overlay(self, overlay: Overlay) -> Self {
        Self {
            overlay: Some(overlay),
            ..self
        }
    }

    #[must_use]
    pub fn without_overlay(self) -> Self {
        Self {
            overlay: None,
            ..self
        }
    }
}

#[cfg(test)]
mod tests;
