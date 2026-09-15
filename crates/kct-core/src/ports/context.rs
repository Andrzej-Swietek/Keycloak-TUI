use std::sync::Arc;

use crate::catalog::Resource;
use crate::path::PathParams;

/// Where a request is addressed: a realm plus whatever parent identifiers the endpoint needs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Context {
    pub realm: Arc<str>,
    pub params: PathParams,
}

impl Context {
    #[must_use]
    pub fn realm(realm: impl AsRef<str>) -> Self {
        let realm = realm.as_ref();
        Self {
            realm: Arc::from(realm),
            params: PathParams::realm(realm),
        }
    }

    /// Server-scoped: no realm in the path (`/admin/serverinfo`, `/admin/realms`).
    #[must_use]
    pub fn server() -> Self {
        Self {
            realm: Arc::from(""),
            params: PathParams::none(),
        }
    }

    #[must_use]
    pub fn with(self, slot: &'static str, value: impl Into<String>) -> Self {
        Self {
            params: self.params.with(slot, value),
            ..self
        }
    }

    /// The parameters for addressing one row of `resource`.
    ///
    /// The row's own identifier deliberately overrides any parent slot of the same name, so
    /// `/groups/{group}/children` can be listed against a parent while each child is still
    /// addressed at `/groups/{group}`.
    #[must_use]
    pub fn for_item(&self, resource: &Resource, id: &str) -> PathParams {
        match resource.item_slot() {
            Some(slot) => self.params.clone().with(slot, id),
            None => self.params.clone(),
        }
    }
}
