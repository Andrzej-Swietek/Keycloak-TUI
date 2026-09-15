//! The pure state machine.
//!
//! `update` performs no I/O and takes no `&mut`: it maps a state and a message to the next
//! state plus a list of effects for the runtime to interpret.

mod actions;
mod attributes;
mod commands;
mod commit;
mod confirm;
mod data;
mod detail;
mod field_edit;
mod keys;
mod ldap;
mod nav;
mod nested;
mod overlay;
mod parse;
mod pick_roles;
mod pick_specs;
mod picker;
mod prompt;
mod raw;
mod save;
mod singleton;
mod specials;
mod spotlight_keys;
mod suggest;
mod tokens;
mod transition;
mod warnings;

pub use detail::visible_fields;
pub use nav::reload;
pub use raw::leaves as raw_leaves;
pub use suggest::matching as suggest_matching;
pub use transition::Transition;

use crossterm::event::KeyEventKind;

use crate::model::Model;
use crate::msg::Msg;

#[must_use]
pub fn update(model: Model, msg: Msg) -> Transition {
    match msg {
        // Some terminals report press *and* release; acting on both doubles every keystroke.
        Msg::Key(key) if key.kind != KeyEventKind::Press => Transition::pure(model),
        Msg::Key(key) => keys::handle(model, key),
        Msg::Tick => Transition::pure(Model {
            ticks: model.ticks.wrapping_add(1),
            ..model.expire_toasts()
        }),
        Msg::Profile(result) => data::profile(model, result),
        Msg::Realms(result) => data::realms(model, result),
        Msg::UserProfile(result) => data::user_profile(model, result),
        Msg::RealmFlags(result) => data::realm_flags(model, result),
        Msg::AuthFlows(result) => data::auth_flows(model, result),
        Msg::Mascots(scan) => data::mascots(model, scan),
        Msg::Rows { token, result } => data::rows(model, token, result),
        Msg::Item { token, result } => data::item(model, token, result),
        Msg::Options { token, result } => picker::loaded(model, token, result),
        Msg::Exclusions { token, result } => picker::excluded(model, token, result),
        Msg::Done { title, result } => data::done(model, title, result),
        Msg::Text { title, result } => data::text(model, title, result),
        Msg::Notice(notice) => Transition::pure(model.noticed(notice)),
        Msg::Logged {
            method,
            path,
            outcome,
        } => Transition::pure(model.logged(crate::model::LogLine {
            at: std::time::Instant::now(),
            method,
            path,
            outcome,
        })),
        Msg::Viewport(rows) => Transition::pure(Model {
            viewport_rows: rows,
            ..model
        }),
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_attributes;
#[cfg(test)]
mod tests_detail;
#[cfg(test)]
mod tests_detail_save;
#[cfg(test)]
mod tests_picker;
#[cfg(test)]
mod tests_picker_clients;
#[cfg(test)]
mod tests_picker_groups;
#[cfg(test)]
mod tests_picker_paths;
#[cfg(test)]
mod tests_session;
#[cfg(test)]
mod tests_support;
