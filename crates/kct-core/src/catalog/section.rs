/// Where a resource lives in the navigation tree. Order here is order on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Section {
    /// Realm-wide configuration: settings, keys, user profile, policies.
    Realm,
    /// Day-to-day subject management: users, groups, organizations.
    Manage,
    /// Things that grant or broker access: clients, scopes, roles, `IdPs`, federation, flows.
    Configure,
    /// Live and historical behaviour: sessions, events, lockouts.
    Observe,
    /// The server itself: version, SPIs, feature flags.
    Server,
    /// Operator workflows that are not a CRUD resource.
    Tools,
}

impl Section {
    pub const ALL: &'static [Self] = &[
        Self::Manage,
        Self::Configure,
        Self::Realm,
        Self::Observe,
        Self::Server,
        Self::Tools,
    ];

    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Realm => "Realm",
            Self::Manage => "Manage",
            Self::Configure => "Configure",
            Self::Observe => "Observe",
            Self::Server => "Server",
            Self::Tools => "Tools",
        }
    }
}
