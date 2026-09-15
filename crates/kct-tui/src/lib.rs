//! Terminal UI for Keycloak, built as a strict Elm architecture.
//!
//! [`model`] is the whole application state; [`update`] is a pure `(Model, Msg) -> Transition`
//! that does no I/O and takes no `&mut`; [`effect`] describes I/O as data for the runtime to
//! interpret; [`view`] renders a `&Model`. Every behavioural rule is therefore a
//! value-in/value-out function with a test next to it.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(
    // `update` is a move-based state machine: taking state by value is the point, and the
    // compiler already proves nothing is left dangling.
    clippy::needless_pass_by_value,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_possible_truncation
)]

pub mod app;
pub mod effect;
pub mod model;
pub mod msg;
pub mod spotlight;
pub mod text_input;
pub mod theme;
pub mod update;
pub mod view;

pub use effect::Effect;
pub use model::Model;
pub use msg::Msg;
pub use text_input::TextInput;
pub use theme::Theme;
