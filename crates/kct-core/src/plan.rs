//! Pure request planning.
//!
//! Turning "this resource, in this context" into a path and query string, for the async
//! client and for the synchronous UI update that describes I/O as data.

use crate::catalog::{Resource, Search};
use crate::error::Result;
use crate::path::ApiPath;
use crate::ports::Context;
use crate::query::Query;

/// A request ready to be sent, with no transport attached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    pub path: ApiPath,
    pub query: Vec<(String, String)>,
}

/// Plan the request that lists one page of a resource.
pub fn list(resource: &Resource, context: &Context, query: &Query) -> Result<Planned> {
    let path = resource.collection.resolve(&context.params)?;

    // Endpoints we filter locally get no server-side paging: paging a list we are about to
    // filter in memory would hide matches that live on the next page.
    let paging = match resource.search {
        Search::None | Search::Client => Query::unbounded().to_pairs(),
        Search::Server | Search::ServerWithAttributes => query.to_pairs(),
    };

    Ok(Planned {
        path,
        query: resource
            .resolved_params(&context.params)
            .into_iter()
            .chain(paging)
            .collect(),
    })
}

/// Plan the request that fetches one object in full.
pub fn item(resource: &Resource, context: &Context, id: &str) -> Result<Planned> {
    let params = context.for_item(resource, id);
    let path = match resource.item {
        Some(template) => template.resolve(&params)?,
        None => resource.collection.resolve(&context.params)?.child(id),
    };

    Ok(Planned {
        path,
        query: resource.resolved_params(&params),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources;

    #[test]
    fn a_locally_filtered_list_asks_for_everything_at_once() {
        let flows = resources::by_id("auth-flows").unwrap();
        let planned = list(flows, &Context::realm("avsystem"), &Query::page(0, 10)).unwrap();

        let max = planned.query.iter().find(|(key, _)| key == "max").unwrap();
        assert_ne!(
            max.1, "10",
            "paging would hide rows the local filter should see"
        );
    }

    #[test]
    fn a_server_filtered_list_forwards_the_operators_query() {
        let users = resources::by_id("users").unwrap();
        let query = Query::page(50, 25).with_search("alice").with_exact(true);
        let planned = list(users, &Context::realm("avsystem"), &query).unwrap();

        assert_eq!(planned.path.as_str(), "/admin/realms/avsystem/users");
        assert!(
            planned
                .query
                .contains(&("first".to_owned(), "50".to_owned()))
        );
        assert!(
            planned
                .query
                .contains(&("search".to_owned(), "alice".to_owned()))
        );
        assert!(
            planned
                .query
                .contains(&("exact".to_owned(), "true".to_owned()))
        );
    }

    #[test]
    fn an_item_is_addressed_with_its_own_id_overriding_the_parent() {
        let children = resources::by_id("group-children").unwrap();
        let context = Context::realm("r").with("group", "parent-id");

        assert_eq!(
            list(children, &context, &Query::new())
                .unwrap()
                .path
                .as_str(),
            "/admin/realms/r/groups/parent-id/children"
        );
        assert_eq!(
            item(children, &context, "child-id").unwrap().path.as_str(),
            "/admin/realms/r/groups/child-id"
        );
    }

    #[test]
    fn a_resource_without_an_item_endpoint_appends_the_id() {
        let credentials = resources::by_id("user-credentials").unwrap();
        let context = Context::realm("r").with("user", "u1");

        assert_eq!(
            item(credentials, &context, "c1").unwrap().path.as_str(),
            "/admin/realms/r/users/u1/credentials/c1"
        );
    }
}
