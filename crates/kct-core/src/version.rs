//! Keycloak version parsing and comparison.
//!
//! [`Version`] is the numeric triple alone — `Copy` and `const`-constructible, so the catalog
//! embeds version gates as constants. Qualifiers such as `-SNAPSHOT` or `.redhat-00001` are
//! display-only and carried elsewhere. [`Version::parse`] yields [`Version::UNKNOWN`] for
//! anything it does not recognise.

use std::fmt;

/// The oldest release this tool claims to understand. Below it the admin API differs enough
/// that pretending otherwise would be dishonest.
pub const SUPPORTED_FLOOR: Version = Version::new(25, 0, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Sentinel for "the server would not tell us". Ordered above everything, so version
    /// gates default to *permitting* the attempt: refusing to show a screen because we could
    /// not read a version number is the worse failure mode.
    pub const UNKNOWN: Self = Self {
        major: u32::MAX,
        minor: 0,
        patch: 0,
    };

    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    #[must_use]
    pub const fn is_unknown(self) -> bool {
        self.major == u32::MAX
    }

    /// Nightly builds report `999.0.0-SNAPSHOT`; they are "newer than everything".
    #[must_use]
    pub const fn is_nightly(self) -> bool {
        self.major >= 900 && !self.is_unknown()
    }

    #[must_use]
    pub fn is_below_floor(self) -> bool {
        !self.is_unknown() && self < SUPPORTED_FLOOR
    }

    /// Version comparison that treats "unknown" as satisfying every lower bound.
    #[must_use]
    pub fn at_least(self, other: Self) -> bool {
        self.is_unknown() || self >= other
    }

    /// `26.6.1` -> `26.6`; used for docs links and the status-bar badge.
    #[must_use]
    pub fn minor_series(self) -> String {
        if self.is_unknown() {
            "unknown".to_owned()
        } else {
            format!("{}.{}", self.major, self.minor)
        }
    }

    /// Parse the numeric prefix of a version string, ignoring any qualifier.
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim().trim_start_matches(['v', 'V']);
        let numeric_len = trimmed
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(trimmed.len());
        let numeric = trimmed[..numeric_len].trim_end_matches('.');

        let mut parts = numeric.split('.');
        let Some(major) = parts.next().and_then(|p| p.parse().ok()) else {
            return Self::UNKNOWN;
        };

        Self {
            major,
            minor: parts.next().and_then(|p| p.parse().ok()).unwrap_or(0),
            patch: parts.next().and_then(|p| p.parse().ok()).unwrap_or(0),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_unknown() {
            f.write_str("unknown")
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// The non-numeric tail of a version string (`SNAPSHOT`, `redhat-00001`), for display only.
///
/// Worth showing: a `-SNAPSHOT` server is a nightly and an operator should know before
/// trusting anything about it.
#[must_use]
pub fn qualifier_of(raw: &str) -> Option<&str> {
    let trimmed = raw.trim().trim_start_matches(['v', 'V']);
    let numeric_len = trimmed
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(trimmed.len());

    Some(trimmed[numeric_len..].trim_start_matches(['-', '.', '_'])).filter(|q| !q.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_shapes_keycloak_actually_reports() {
        assert_eq!(Version::parse("26.6.1"), Version::new(26, 6, 1));
        assert_eq!(Version::parse("26.0.0").minor_series(), "26.0");
        assert_eq!(Version::parse("25.0"), Version::new(25, 0, 0));

        let nightly = Version::parse("999.0.0-SNAPSHOT");
        assert!(nightly.is_nightly());
        assert_eq!(qualifier_of("999.0.0-SNAPSHOT"), Some("SNAPSHOT"));

        // Red Hat build of Keycloak: a fourth component lives in the qualifier.
        assert_eq!(
            Version::parse("26.7.0.redhat-00001"),
            Version::new(26, 7, 0)
        );
        assert_eq!(qualifier_of("26.7.0.redhat-00001"), Some("redhat-00001"));
        assert_eq!(qualifier_of("26.6.1"), None);
    }

    #[test]
    fn garbage_degrades_to_unknown_which_permits_everything() {
        let unknown = Version::parse("not-a-version");
        assert!(unknown.is_unknown());
        assert!(unknown.at_least(Version::new(26, 0, 0)));
        assert!(!unknown.is_below_floor());
        assert_eq!(unknown.to_string(), "unknown");
    }

    #[test]
    fn ordering_ignores_qualifiers_but_respects_the_floor() {
        assert!(Version::parse("26.0.0") > Version::parse("25.9.9"));
        assert_eq!(Version::parse("26.6.1-SNAPSHOT"), Version::parse("26.6.1"));
        assert!(Version::parse("24.0.5").is_below_floor());
        assert!(!Version::parse("25.0.0").is_below_floor());
    }
}
