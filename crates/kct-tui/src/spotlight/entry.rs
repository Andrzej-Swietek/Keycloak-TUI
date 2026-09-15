use kct_core::capability::Support;

use super::command::Command;

/// One row of the palette.
#[derive(Debug, Clone)]
pub struct Entry {
    pub command: Command,
    pub title: String,
    pub subtitle: String,
    /// Shown on the right; explains a disabled entry or flags a preview feature.
    pub note: Option<String>,
    /// Disabled entries stay visible: "Organizations — needs Keycloak 26" teaches the
    /// operator something, whereas a missing row just looks like a bug.
    pub available: bool,
    pub(super) haystack: String,
}

impl Entry {
    pub(super) fn new(command: Command, title: String, subtitle: String, keywords: &str) -> Self {
        let haystack = format!("{title} {subtitle} {keywords}").to_lowercase();
        Self {
            command,
            title,
            subtitle,
            note: None,
            available: true,
            haystack,
        }
    }

    pub(super) fn gated(self, support: &Support) -> Self {
        match support {
            // An unknown answer stays offered: the server has the final word.
            Support::Available | Support::Unknown(_) => self,
            Support::Unavailable(why) => Self {
                available: false,
                note: Some(why.clone()),
                ..self
            },
        }
    }
}
