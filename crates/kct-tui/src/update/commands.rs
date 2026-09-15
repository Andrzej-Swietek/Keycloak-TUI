use crate::effect::Effect;
use crate::model::{List, Model, Overlay, Spotlight};
use crate::spotlight;

use super::{nav, transition::Transition};

pub(super) fn copy_selection(model: Model) -> Transition {
    match model.list.selected() {
        Some(row) => {
            let text = kct_core::json::pretty(row);
            Transition::with(
                model,
                Effect::Copy {
                    text,
                    label: "row JSON".to_owned(),
                },
            )
        }
        None => Transition::pure(model),
    }
}

pub(super) fn paged(model: Model, forward: bool) -> Transition {
    let query = if forward {
        model.list.query.clone().next_page()
    } else {
        model.list.query.clone().previous_page()
    };

    nav::reload(Model {
        list: List {
            query,
            ..model.list
        },
        ..model
    })
}

pub(super) fn open_palette(model: Model) -> Transition {
    let entries = spotlight::catalog(
        &model.session.profile,
        &model.session.realms,
        Some(model.resource()),
    );
    Transition::pure(model.with_overlay(Overlay::Spotlight(Spotlight::new(entries))))
}
