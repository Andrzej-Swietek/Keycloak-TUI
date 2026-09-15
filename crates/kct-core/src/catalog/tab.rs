use super::field::Field;

/// One tab of a detail view.
#[derive(Debug, Clone, Copy)]
pub enum Tab {
    /// A declarative form over the representation.
    Form {
        title: &'static str,
        fields: &'static [Field],
    },
    /// The raw representation, viewable and editable. Present on every resource: the escape
    /// hatch that guarantees a new Keycloak field is never unreachable from this tool.
    Raw,
    /// A nested collection, rendered by the same engine one level down.
    ///
    /// The only composition primitive, and enough for every sub-collection Keycloak exposes.
    Nested {
        title: &'static str,
        /// Catalog id of the child resource.
        resource: &'static str,
        /// Path slot of the child that the parent's identifier fills.
        slot: &'static str,
        /// Which field of the parent supplies that value. `None` uses the parent's key.
        ///
        /// Needed where an endpoint is addressed by a different field than the object is
        /// keyed on: flows are keyed by `id` but their executions live under `alias`.
        source: Option<&'static str>,
        /// Constant slots the parent pins for the child, such as the `ownerKind` that tells
        /// one role-mapping endpoint whether it is serving a user or a group.
        preset: &'static [(&'static str, &'static str)],
    },
}

impl Tab {
    #[must_use]
    pub const fn title(&self) -> &'static str {
        match self {
            Self::Form { title, .. } | Self::Nested { title, .. } => title,
            Self::Raw => "JSON",
        }
    }
}
