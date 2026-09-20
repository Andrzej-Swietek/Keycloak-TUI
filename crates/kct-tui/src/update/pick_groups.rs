//! Picking where a group goes.

use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, Result};

use crate::model::{Commit, Model};

use super::pick_spec::Spec;

/// Groups the selected group can be moved under.
///
/// Errs with nothing selected. The group's own representation is what travels: Keycloak
/// rejects a move body without the name.
pub(super) fn move_group(model: &Model) -> Result<Spec> {
    let (id, label) = super::actions::selection(model)
        .ok_or_else(|| kct_core::Error::config("select a group first"))?;
    let representation = super::specials::current_object(model)
        .or_else(|| model.list.selected().cloned())
        .ok_or_else(|| kct_core::Error::config("the group's representation is not loaded"))?;

    let params = PathParams::realm(&*model.session.realm);
    let groups = ep::GROUPS.resolve(&params)?;

    Ok(Spec::from(
        format!("Move {label} under"),
        groups.clone(),
        &["/path", "/name"],
        Commit::EachUnder {
            method: Method::Post,
            base: groups,
            key: "id",
            suffix: "children",
            body: representation,
            title: format!("moved {label}"),
        },
    )
    // A group cannot become its own parent.
    .without(vec![id]))
}
