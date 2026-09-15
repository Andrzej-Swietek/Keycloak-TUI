use std::sync::Arc;

use kct_core::Error;
use serde_json::Value;

mod detect;

pub use detect::{auth_flows, mascots, profile, realm_flags, realms, user_profile};

use crate::model::{Detail, List, Model, Overlay};
use crate::msg::Notice;

use super::singleton::open_singleton;
use super::{nav, transition::Transition};

pub fn rows(model: Model, token: u64, result: Result<crate::msg::Page, Error>) -> Transition {
    if token != model.list.token {
        return Transition::pure(model);
    }

    let resource = model.resource();
    let viewport = model.viewport_rows;

    match result {
        Ok(page) => {
            let list = List {
                rows: page.rows,
                envelope: page.envelope,
                loading: false,
                error: None,
                ..model.list
            }
            .refiltered(resource)
            .scrolled_into_view(viewport);

            let model = Model { list, ..model };

            // A singleton has nothing to pick from; show the object itself.
            if resource.is_singleton() {
                open_singleton(model)
            } else {
                Transition::pure(model)
            }
        }
        Err(error) => Transition::pure(Model {
            list: List {
                loading: false,
                error: Some(error),
                ..model.list
            },
            ..model
        }),
    }
}

pub fn item(model: Model, token: u64, result: Result<Value, Error>) -> Transition {
    let Some(detail) = &model.detail else {
        return Transition::pure(model);
    };
    if detail.token != token {
        return Transition::pure(model);
    }

    let detail = detail.clone();
    match result {
        Ok(object) => Transition::pure(Model {
            detail: Some(Detail {
                original: object.clone(),
                draft: object,
                loading: false,
                error: None,
                ..detail
            }),
            ..model
        }),
        Err(error) => Transition::pure(Model {
            detail: Some(Detail {
                loading: false,
                error: Some(error),
                ..detail
            }),
            ..model
        }),
    }
}

/// A write finished. Success re-reads the list so the screen matches the server.
pub fn done(model: Model, title: String, result: Result<Option<String>, Error>) -> Transition {
    match result {
        Ok(_) => refresh(model.noticed(Notice::success(title))),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(&title, &error))),
    }
}

/// Reload the listing, reopening whatever was under edit unless it asked to close.
fn refresh(model: Model) -> Transition {
    let reopen = model
        .detail
        .as_ref()
        .filter(|detail| detail.after_save == crate::model::AfterSave::Reopen)
        .map(|detail| (detail.id.clone(), detail.title.to_string()))
        .filter(|(id, _)| !id.is_empty());

    match reopen {
        Some((id, title)) => nav::reload(model).and_then(|model| nav::open_item(model, id, title)),
        None => nav::reload(model),
    }
}

pub fn text(model: Model, title: String, result: Result<String, Error>) -> Transition {
    match result {
        Ok(body) => Transition::pure(model.with_overlay(Overlay::Viewer {
            title: Arc::from(title),
            body: Arc::from(body),
            scroll: 0,
        })),
        Err(error) => Transition::pure(model.noticed(Notice::from_error(&title, &error))),
    }
}
