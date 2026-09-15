//! Every colour the UI uses, named once.
//!
//! Widgets ask [`Theme`] for a role — [`Theme::danger`], [`Theme::dim`], [`Theme::selected`]
//! — and never name a colour themselves. Only the 16 ANSI colours are used, so the palette
//! follows whatever theme the operator's terminal is set to.

use kct_core::catalog::Tone;
use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub accent: Color,
    pub text: Color,
    pub muted: Color,
    pub good: Color,
    pub warn: Color,
    pub bad: Color,
    pub border: Color,
    pub border_focused: Color,
    pub selection_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: Color::Cyan,
            text: Color::Reset,
            muted: Color::DarkGray,
            good: Color::Green,
            warn: Color::Yellow,
            bad: Color::Red,
            border: Color::DarkGray,
            border_focused: Color::Cyan,
            selection_bg: Color::Indexed(238),
        }
    }
}

impl Theme {
    #[must_use]
    pub fn base(&self) -> Style {
        Style::default().fg(self.text)
    }

    #[must_use]
    pub fn dim(&self) -> Style {
        Style::default().fg(self.muted)
    }

    #[must_use]
    pub fn title(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    #[must_use]
    pub fn selected(&self) -> Style {
        Style::default()
            .bg(self.selection_bg)
            .add_modifier(Modifier::BOLD)
    }

    #[must_use]
    pub fn border_style(&self, focused: bool) -> Style {
        Style::default().fg(if focused {
            self.border_focused
        } else {
            self.border
        })
    }

    #[must_use]
    pub fn danger(&self) -> Style {
        Style::default().fg(self.bad).add_modifier(Modifier::BOLD)
    }

    /// Style for a table cell, given the column's semantic tone and its rendered value.
    ///
    /// [`Tone::Flag`] reads the value: an `enabled` column should be green when on and red
    /// when off without every call site restating that.
    #[must_use]
    pub fn cell(&self, tone: Tone, value: &str) -> Style {
        match tone {
            Tone::Primary => Style::default()
                .fg(self.accent)
                .add_modifier(Modifier::BOLD),
            Tone::Normal => self.base(),
            Tone::Muted => self.dim(),
            Tone::Good => Style::default().fg(self.good),
            Tone::Warn => Style::default().fg(self.warn),
            Tone::Bad => Style::default().fg(self.bad),
            Tone::Flag => Style::default().fg(match value {
                "yes" | "on" | "true" | "ON" | "ACTIVE" | "ENABLED" => self.good,
                "" => self.muted,
                _ => self.bad,
            }),
        }
    }

    /// Colour for a feature-flag support classifier.
    #[must_use]
    pub fn support(&self, classifier: &str) -> Style {
        Style::default().fg(match classifier {
            "DEFAULT" => self.good,
            "DEPRECATED" => self.bad,
            "PREVIEW" | "PREVIEW_DISABLED_BY_DEFAULT" | "EXPERIMENTAL" => self.warn,
            _ => self.muted,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_columns_colour_themselves_from_their_value() {
        let theme = Theme::default();
        assert_eq!(theme.cell(Tone::Flag, "yes").fg, Some(theme.good));
        assert_eq!(theme.cell(Tone::Flag, "DISABLED").fg, Some(theme.bad));
        assert_eq!(theme.cell(Tone::Flag, "").fg, Some(theme.muted));
    }

    #[test]
    fn preview_and_deprecated_features_are_visually_distinct() {
        let theme = Theme::default();
        assert_eq!(theme.support("DEFAULT").fg, Some(theme.good));
        assert_eq!(theme.support("PREVIEW").fg, Some(theme.warn));
        assert_eq!(theme.support("DEPRECATED").fg, Some(theme.bad));
    }
}
