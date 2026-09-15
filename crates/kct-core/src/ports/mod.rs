//! Ports: the boundary between "what Keycloak means" and "how bytes move".
//!
//! Only [`Gateway`] has to be implemented to plug in a transport. Everything else — listing a
//! catalog resource, a lossless save, capability detection — is a default method on [`Rest`]
//! and [`AdminApi`], blanket-implemented for every `Gateway`. A test double is one method yet
//! exercises the entire API surface.

mod admin;
mod context;
mod gateway;
mod request;
mod rest;
mod tokens;

pub use admin::AdminApi;
pub use context::Context;
pub use gateway::Gateway;
pub use request::{Expect, Request, Response};
pub use rest::Rest;
pub use tokens::TokenSource;

#[cfg(test)]
mod tests;
