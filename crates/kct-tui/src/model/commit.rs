use kct_core::{ApiPath, Method};
use serde_json::Value;

/// The request shape a picker's selection turns into.
///
/// Keycloak varies by endpoint: role mappings take one array body, group membership and
/// client scopes take one request per object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Commit {
    /// Choosing does not write anything: it opens a second picker, parameterised by what was
    /// chosen. Assigning a client role needs the client first, exactly as the console does.
    Chain {
        next: &'static str,
        slot: &'static str,
        key: &'static str,
    },
    /// Send every chosen object at once as a JSON array.
    Batch {
        method: Method,
        path: ApiPath,
        title: String,
    },
    /// Send one object with a single field replaced by what was chosen. Authentication
    /// executions work this way: the whole execution goes back with a new `requirement`.
    Replace {
        method: Method,
        path: ApiPath,
        base: Value,
        field: &'static str,
        from: &'static str,
        title: String,
    },
    /// One request per chosen object, each carrying `{field: chosen[from]}` as its body.
    /// Adding steps to an authentication flow works this way.
    EachBody {
        method: Method,
        path: ApiPath,
        field: &'static str,
        from: &'static str,
        title: String,
    },
    /// Choosing writes nothing: it asks for a name, to create under the chosen type. An
    /// authorization policy's type is part of its URL, including one a deployed SPI adds.
    Name {
        base: ApiPath,
        from: &'static str,
        title: &'static str,
        singular: &'static str,
    },
    /// One request per chosen object, sent to a sub-path of it with a fixed body. Moving a
    /// group posts the group's own representation under the chosen parent.
    EachUnder {
        method: Method,
        base: ApiPath,
        key: &'static str,
        suffix: &'static str,
        body: Value,
        title: String,
    },
    /// One request per chosen object, addressed by its id.
    Each {
        method: Method,
        base: ApiPath,
        key: &'static str,
        title: String,
    },
}
