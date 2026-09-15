//! Pure domain core for the Keycloak TUI.
//!
//! This crate knows what Keycloak *is*; it knows nothing about HTTP, terminals or files.
//! Everything here is either data, a pure function over data, or a port (trait) that an
//! adapter crate implements. That split is what keeps the UI testable without a server and
//! the client swappable without touching the UI.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_debug_implementations)]
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod capability;
pub mod catalog;
pub mod doctor;
pub mod error;
pub mod json;
pub mod jwt;
pub mod page;
pub mod path;
pub mod plan;
pub mod ports;
pub mod query;
pub mod resources;
pub mod secret;
pub mod version;

pub use capability::{Requirement, ServerProfile, Support};
pub use error::{AuthError, Error, Method, Result};
pub use page::Page;
pub use path::{ApiPath, PathParams, Template};
pub use ports::{AdminApi, Context, Gateway, Request, Response, Rest, TokenSource};
pub use query::Query;
pub use secret::Secret;
pub use version::Version;
