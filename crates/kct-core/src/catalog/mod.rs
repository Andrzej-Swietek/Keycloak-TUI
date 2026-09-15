//! Every screen the tool offers, described as data.
//!
//! One [`Resource`] value per Keycloak collection says where it lives ([`crate::path::Template`]), how the
//! response is shaped ([`Shape`]), which field addresses an item ([`Key`]), what the listing
//! and the forms show ([`Column`], [`Tab`], [`Field`]), what may be done to it ([`Ops`],
//! [`Action`]) and which servers have it ([`crate::capability::Requirement`]). One engine renders them all, so a
//! new screen is a value in [`crate::resources::CATALOG`] and nothing else.
//!
//! [`crate::doctor`] checks these values against a running server.

mod action;
mod column;
mod encoding;
mod field;
pub mod new_object;
mod ops;
mod resource;
mod section;
mod shape;
mod tab;

pub use action::{Action, Danger, Verb};
pub use column::{Column, Projection, Tone, Width};
pub use encoding::{Encoding, format_seconds};
pub use field::{Field, FieldKind};
pub use new_object::NewObject;
pub use ops::{Key, Ops, Search};
pub use resource::{FixedParam, Resource};
pub use section::Section;
pub use shape::Shape;
pub use tab::Tab;

#[cfg(test)]
mod tests;
