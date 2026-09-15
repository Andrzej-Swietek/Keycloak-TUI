use std::collections::BTreeMap;

/// Values substituted into a [`super::Template`].
///
/// `realm` is modelled explicitly because almost everything needs it and forgetting it is
/// the most likely mistake; everything else is a named slot.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PathParams {
    slots: BTreeMap<&'static str, String>,
}

impl PathParams {
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn realm(realm: impl Into<String>) -> Self {
        Self::none().with("realm", realm)
    }

    #[must_use]
    pub fn with(self, name: &'static str, value: impl Into<String>) -> Self {
        let mut slots = self.slots;
        slots.insert(name, value.into());
        Self { slots }
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.slots.get(name).map(String::as_str)
    }

    /// Slot names present here, for diagnostics when resolution fails.
    #[must_use]
    pub fn names(&self) -> Vec<&'static str> {
        self.slots.keys().copied().collect()
    }
}
