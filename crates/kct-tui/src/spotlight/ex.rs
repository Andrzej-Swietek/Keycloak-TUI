/// Vim-style commands typed after `:`.
///
/// The palette and the command line are the same prompt: anything that parses as one of
/// these runs as a command, anything else is fuzzy-matched against the palette. That way
/// `:w` works without giving up search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ex {
    /// Save the open object.
    Write,
    /// Leave the current view, refusing if there are unsaved changes.
    Quit,
    /// Leave, discarding unsaved changes.
    QuitForce,
    /// Save, then leave.
    WriteQuit,
    /// Re-fetch.
    Edit,
    /// Leave the application.
    QuitAll,
}

impl Ex {
    #[must_use]
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim() {
            "w" => Some(Self::Write),
            "q" => Some(Self::Quit),
            "q!" => Some(Self::QuitForce),
            "wq" | "x" => Some(Self::WriteQuit),
            "e" | "e!" => Some(Self::Edit),
            "qa" | "qa!" | "quit" => Some(Self::QuitAll),
            _ => None,
        }
    }

    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Write => "save the open object",
            Self::Quit => "close this view (refuses if unsaved)",
            Self::QuitForce => "close this view, discarding changes",
            Self::WriteQuit => "save, then close",
            Self::Edit => "re-fetch from the server",
            Self::QuitAll => "leave kct",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_commands_a_vim_user_will_reach_for_all_parse() {
        assert_eq!(Ex::parse("w"), Some(Ex::Write));
        assert_eq!(Ex::parse("q"), Some(Ex::Quit));
        assert_eq!(Ex::parse("q!"), Some(Ex::QuitForce));
        assert_eq!(Ex::parse("wq"), Some(Ex::WriteQuit));
        assert_eq!(Ex::parse("x"), Some(Ex::WriteQuit));
        assert_eq!(
            Ex::parse(" w "),
            Some(Ex::Write),
            "stray spaces are forgiven"
        );
    }

    #[test]
    fn anything_else_falls_through_to_the_palette() {
        assert_eq!(Ex::parse("users"), None);
        assert_eq!(Ex::parse("write"), None);
        assert_eq!(Ex::parse(""), None);
    }
}
