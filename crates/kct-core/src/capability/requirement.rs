use crate::version::Version;

/// A declarative precondition attached to a screen, action or field.
///
/// `Copy` and `const`-constructible, so the whole resource catalog can embed gates as plain
/// constants that cost nothing at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    Always,
    MinVersion(Version),
    BelowVersion(Version),
    Feature(&'static str),
    Provider { spi: &'static str, id: &'static str },
    All(&'static [Requirement]),
    Any(&'static [Requirement]),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Support {
    Available,
    /// Not available, carrying the reason to show the operator.
    Unavailable(String),
    /// We could not tell. The UI offers it anyway and lets the server decide.
    Unknown(String),
}

impl Support {
    /// Should the UI let the operator attempt this?
    #[must_use]
    pub const fn is_permitted(&self) -> bool {
        matches!(self, Self::Available | Self::Unknown(_))
    }

    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Available => None,
            Self::Unavailable(why) | Self::Unknown(why) => Some(why),
        }
    }

    /// Conjunction: the most pessimistic answer wins.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unavailable(why), _) | (_, Self::Unavailable(why)) => Self::Unavailable(why),
            (Self::Unknown(why), _) | (_, Self::Unknown(why)) => Self::Unknown(why),
            (Self::Available, Self::Available) => Self::Available,
        }
    }

    /// Disjunction: the most optimistic answer wins.
    #[must_use]
    pub fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::Available, _) | (_, Self::Available) => Self::Available,
            (Self::Unknown(why), _) | (_, Self::Unknown(why)) => Self::Unknown(why),
            (Self::Unavailable(why), Self::Unavailable(_)) => Self::Unavailable(why),
        }
    }
}
