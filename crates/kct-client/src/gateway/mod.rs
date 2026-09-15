//! The HTTP adapter: the one place in this workspace that knows about sockets.

mod client;
mod errors;
mod transport;

pub use client::KeycloakClient;
pub use transport::HttpTransport;

#[cfg(test)]
mod tests;
