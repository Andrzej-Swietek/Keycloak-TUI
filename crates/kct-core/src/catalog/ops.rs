use serde_json::Value;

/// Which operations a resource supports. A hand-rolled bitset keeps the catalog `const`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ops(u8);

impl Ops {
    pub const LIST: Self = Self(1 << 0);
    pub const READ: Self = Self(1 << 1);
    pub const CREATE: Self = Self(1 << 2);
    pub const UPDATE: Self = Self(1 << 3);
    pub const DELETE: Self = Self(1 << 4);

    /// Listable and inspectable, never written.
    pub const RO: Self = Self(Self::LIST.0 | Self::READ.0);
    /// The full CRUD set.
    pub const CRUD: Self = Self(Self::RO.0 | Self::CREATE.0 | Self::UPDATE.0 | Self::DELETE.0);
    /// Everything but creation: singletons such as realm settings.
    pub const RW: Self = Self(Self::RO.0 | Self::UPDATE.0);

    #[must_use]
    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[must_use]
    pub const fn has(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

/// Which JSON field of a row identifies the object in its item URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key(pub &'static str);

impl Key {
    pub const ID: Self = Self("id");
    pub const NAME: Self = Self("name");
    pub const ALIAS: Self = Self("alias");

    /// Any other field name: Authorization Services uses `_id`, realms use `realm`.
    #[must_use]
    pub const fn custom(field: &'static str) -> Self {
        Self(field)
    }

    #[must_use]
    pub fn of(self, row: &Value) -> Option<&str> {
        row.get(self.0).and_then(Value::as_str)
    }
}

/// How a list is filtered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Search {
    /// The endpoint honours `search=`; filtering happens on the server.
    Server,
    /// The endpoint returns everything; we filter the fetched rows locally.
    Client,
    /// `search=` plus `q=` attribute queries (users, groups).
    ServerWithAttributes,
    /// No filtering at all.
    None,
}
