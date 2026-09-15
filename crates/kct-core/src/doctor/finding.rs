use std::fmt;

/// How much a piece of drift matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Something the catalog claims that this server does not do. Fails a check run.
    Broken,
    /// The server offers something the catalog does not surface. Reachable through the JSON
    /// tab, so nothing is lost — but a form is now less complete than it was.
    Missing,
    /// Worth knowing, not worth failing a build over.
    Note,
}

impl Severity {
    #[must_use]
    pub const fn marker(self) -> &'static str {
        match self {
            Self::Broken => "BROKEN ",
            Self::Missing => "missing",
            Self::Note => "note   ",
        }
    }

    /// Whether this should fail a check run.
    #[must_use]
    pub const fn is_fatal(self) -> bool {
        matches!(self, Self::Broken)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub severity: Severity,
    /// Catalog id the finding belongs to.
    pub resource: String,
    pub detail: String,
}

impl Finding {
    #[must_use]
    pub fn broken(resource: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            severity: Severity::Broken,
            resource: resource.into(),
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn missing(resource: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            severity: Severity::Missing,
            resource: resource.into(),
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn note(resource: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            severity: Severity::Note,
            resource: resource.into(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}  {:<26} {}",
            self.severity.marker(),
            self.resource,
            self.detail
        )
    }
}

/// Everything one check run turned up.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub findings: Vec<Finding>,
}

impl Report {
    #[must_use]
    pub fn with(mut self, findings: impl IntoIterator<Item = Finding>) -> Self {
        self.findings.extend(findings);
        self
    }

    /// Sorted worst-first, then by resource, so a diff between runs is readable.
    #[must_use]
    pub fn sorted(mut self) -> Self {
        self.findings.sort_by(|a, b| {
            a.severity
                .cmp(&b.severity)
                .then(a.resource.cmp(&b.resource))
        });
        self
    }

    #[must_use]
    pub fn count(&self, severity: Severity) -> usize {
        self.findings
            .iter()
            .filter(|finding| finding.severity == severity)
            .count()
    }

    #[must_use]
    pub fn is_healthy(&self) -> bool {
        !self
            .findings
            .iter()
            .any(|finding| finding.severity.is_fatal())
    }
}
