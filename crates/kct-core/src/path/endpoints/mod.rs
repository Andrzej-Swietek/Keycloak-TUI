//! The canonical endpoint set, grouped the way Keycloak's own resource tree is.

mod clients;
mod observe;
mod realm;
mod security;
mod subjects;

pub use clients::*;
pub use observe::*;
pub use realm::*;
pub use security::*;
pub use subjects::*;
