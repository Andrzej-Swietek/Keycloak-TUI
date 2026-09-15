use kct_core::catalog::Danger;

use crate::model::{Model, Overlay};
use crate::msg::Level;

/// What the mascot is reacting to.
///
/// Ordered by how much it should override: an operator about to delete a realm needs the
/// alarm, not a leftover smile from the previous save.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    /// A destructive confirmation is open.
    Alarmed,
    /// Something failed.
    Worried,
    /// A request is in flight.
    Busy,
    /// A write just succeeded.
    Happy,
    /// Dry-run: watching, writing nothing.
    Sly,
    Idle,
}

impl Mood {
    #[must_use]
    pub fn of(model: &Model) -> Self {
        if let Some(Overlay::Confirm(confirm)) = &model.overlay
            && confirm.danger == Danger::Destructive
        {
            return Self::Alarmed;
        }
        if model.list.error.is_some()
            || model
                .toasts
                .last()
                .is_some_and(|toast| toast.notice.level == Level::Failure)
        {
            return Self::Worried;
        }
        if model.list.loading || model.detail.as_ref().is_some_and(|detail| detail.loading) {
            return Self::Busy;
        }
        if model
            .toasts
            .last()
            .is_some_and(|toast| toast.notice.level == Level::Success)
        {
            return Self::Happy;
        }
        if model.session.dry_run {
            return Self::Sly;
        }
        Self::Idle
    }

    /// One line saying *why* the face looks like that.
    ///
    /// The panel is landscape, so there is room beside the sprite. Filling it with the
    /// reason turns a decoration into something worth glancing at.
    #[must_use]
    pub fn detail(self, model: &Model) -> String {
        match self {
            Self::Alarmed => "type the name to confirm".to_owned(),
            Self::Worried => model
                .list
                .error
                .as_ref()
                .map(ToString::to_string)
                .or_else(|| model.toasts.last().map(|toast| toast.notice.text.clone()))
                .unwrap_or_else(|| "something went wrong".to_owned())
                .lines()
                .next()
                .unwrap_or_default()
                .to_owned(),
            Self::Busy => format!("loading {}", model.resource().plural.to_lowercase()),
            Self::Happy => model.toasts.last().map_or_else(
                || "saved".to_owned(),
                |toast| toast.notice.text.clone(),
            ),
            Self::Sly => "writes are suppressed".to_owned(),
            Self::Idle => format!(
                "{} {}",
                model.list.len(),
                model.resource().plural.to_lowercase()
            ),
        }
    }

    #[must_use]
    pub const fn caption(self) -> &'static str {
        match self {
            Self::Alarmed => "careful!",
            Self::Worried => "hmm...",
            Self::Busy => "working",
            Self::Happy => "done!",
            Self::Sly => "dry-run",
            Self::Idle => "ready",
        }
    }
}
