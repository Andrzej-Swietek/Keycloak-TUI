use std::sync::Arc;

use kct_core::catalog::Resource;
use kct_core::{Context, plan};

use crate::effect::Effect;
use crate::model::{Detail, List, Model};
use crate::msg::{Fetch, Notice};

use super::transition::Transition;

/// Re-fetch the current listing.
#[must_use]
pub fn reload(model: Model) -> Transition {
    let resource = model.resource();

    // Gate here rather than only on navigation: `--screen` and a realm switch reach a view
    // without passing through `go_to`, and a request we know will 404 helps nobody.
    if let Some(why) = model
        .session
        .support_for(resource)
        .reason()
        .filter(|_| !model.session.support_for(resource).is_permitted())
    {
        let why = why.to_owned();
        return Transition::pure(Model {
            list: List {
                loading: false,
                error: Some(kct_core::Error::unsupported(resource.plural, why)),
                ..List::default()
            },
            detail: None,
            ..model
        });
    }
    let context = model.context().clone();
    let query = model.list.query.clone();
    let (model, token) = model.issue_token();

    match plan::list(resource, &context, &query) {
        Ok(planned) => Transition::with(
            Model {
                list: List::loading(token, query),
                detail: None,
                ..model
            },
            Effect::Load {
                token,
                fetch: Fetch::Rows,
                path: planned.path,
                query: planned.query,
                shape: resource.shape,
            },
        ),
        Err(error) => Transition::pure(model.noticed(Notice::from_error("cannot list", &error))),
    }
}

/// Replace the navigation stack with a top-level screen.
#[must_use]
pub fn go_to(model: Model, resource: &'static Resource) -> Transition {
    let support = model.session.support_for(resource);
    if !support.is_permitted() {
        let why = support.reason().unwrap_or("not available").to_owned();
        return Transition::pure(
            model.noticed(Notice::warning(format!("{}: {why}", resource.plural))),
        );
    }

    let context = Context::realm(&*model.session.realm);
    reload(Model {
        nav: model.nav.jumped(resource, context),
        ..model
    })
}

/// Open the highlighted row in the detail pane.
#[must_use]
pub fn open_selected(model: Model) -> Transition {
    let resource = model.resource();
    let Some(row) = model.list.selected().cloned() else {
        return Transition::pure(model);
    };
    let Some(id) = resource.key_of(&row).map(ToOwned::to_owned) else {
        return Transition::pure(model.noticed(Notice::warning(format!(
            "this {} has no '{}' to address it by",
            resource.singular, resource.key.0
        ))));
    };

    let title = resource.label_of(&row);
    if resource.item.is_none() {
        return Transition::pure(Model {
            detail: Some(Detail::loaded(id, title, row)),
            ..model
        });
    }
    open_item(model, id, title)
}

/// Open the object `id` of the current screen, fetching it in full.
///
/// Leaves the model untouched and reports the failure as a notice when the path cannot be
/// planned.
#[must_use]
pub fn open_item(model: Model, id: String, title: String) -> Transition {
    let resource = model.resource();
    let context = model.context().clone();
    let (model, token) = model.issue_token();

    match plan::item(resource, &context, &id) {
        Ok(planned) => Transition::with(
            Model {
                detail: Some(Detail::loading(id, title, token)),
                ..model
            },
            Effect::Load {
                token,
                fetch: Fetch::Item,
                path: planned.path,
                query: planned.query,
                shape: kct_core::catalog::Shape::Singleton,
            },
        ),
        Err(error) => Transition::pure(model.noticed(Notice::from_error("cannot open", &error))),
    }
}

/// Leave the current view: close the detail, or climb one level.
#[must_use]
pub fn back(model: Model) -> Transition {
    if model.detail.is_some() {
        return Transition::pure(Model {
            detail: None,
            ..model
        });
    }

    match model.nav.clone().popped() {
        Some(nav) => reload(Model { nav, ..model }),
        None => Transition::pure(model),
    }
}

/// Point the whole session at another realm.
#[must_use]
pub fn switch_realm(model: Model, realm: String) -> Transition {
    let resource = model.resource();
    let context = Context::realm(&realm);

    let model = Model {
        session: crate::model::Session {
            realm: Arc::from(realm.as_str()),
            ..model.session
        },
        nav: model.nav.rerooted(resource, context),
        ..model
    };

    reload(model.noticed(Notice::info(format!("realm: {realm}"))))
}
