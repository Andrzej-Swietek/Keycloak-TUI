//! HTTP adapter for the Keycloak admin API.
//!
//! Implements the single [`kct_core::Gateway`] port. Three properties are deliberate:
//! redirects are never followed (a 3xx would re-send the bearer token or grant password),
//! error bodies are capped before being logged, and writes are never retried automatically.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_debug_implementations)]
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod auth;
pub mod config;
pub mod gateway;

pub use config::{Connection, Credentials, Timeouts, TlsOptions};
pub use gateway::{HttpTransport, KeycloakClient};
