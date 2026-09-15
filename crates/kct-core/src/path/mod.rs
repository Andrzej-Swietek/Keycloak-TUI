//! Every admin-API URL this tool can build.
//!
//! Endpoint paths are the part of Keycloak most likely to move between releases. Keeping
//! them behind templates resolved here makes a relocated endpoint a one-line change.

mod api_path;
pub mod endpoints;
mod params;
mod template;

pub use api_path::{ApiPath, encode_segment};
pub use params::PathParams;
pub use template::Template;

#[cfg(test)]
mod tests;
