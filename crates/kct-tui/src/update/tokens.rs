use std::sync::Arc;

use kct_core::path::{PathParams, endpoints as ep};
use kct_core::{Method, plan};

use crate::effect::Effect;
use crate::model::{Model, Overlay};
use crate::msg::Notice;

use super::transition::Transition;

/// Copy the bearer the session is presenting. Useful verbatim in `curl` or a REST client.
#[must_use]
pub fn copy_token(model: Model) -> Transition {
    Transition::with(model, Effect::CopyToken)
}

/// Decode the session's own token and show what the server sees.
#[must_use]
pub fn inspect(model: Model) -> Transition {
    Transition::with(model, Effect::InspectToken)
}

/// Reproduce the most recent admin call as a `curl` command on the clipboard.
///
/// The bearer is referenced as `$KC_TOKEN`; the command carries no credential.
#[must_use]
pub fn copy_curl(model: Model) -> Transition {
    let Some(last) = model.log.back() else {
        return Transition::pure(model.noticed(Notice::info("no request to reproduce yet")));
    };

    let effect = Effect::CopyCurl {
        method: last.method,
        path: last.path.clone(),
        base_url: model.session.base_url.to_string(),
    };
    Transition::with(model, effect)
}

/// Obtain a token *as the selected client*, so an operator can see what that client really
/// gets — the usual answer to "the app says my role is missing".
#[must_use]
pub fn token_for_client(model: Model) -> Transition {
    let Some((id, client_id)) = super::actions::selection(&model) else {
        return Transition::pure(model.noticed(Notice::warning("no client selected")));
    };

    let planned = match plan::item(model.resource(), model.context(), &id) {
        Ok(planned) => planned,
        Err(error) => {
            return Transition::pure(
                model.noticed(Notice::from_error("cannot read client", &error)),
            );
        }
    };

    Transition::with(
        model.clone(),
        Effect::ClientToken {
            realm: model.session.realm.to_string(),
            client_id,
            secret_path: planned.path.sub("client-secret"),
        },
    )
}

/// Show the realm's JWKS.
#[must_use]
pub fn realm_keys(model: Model) -> Transition {
    match ep::OIDC_CERTS.resolve(&PathParams::realm(&*model.session.realm)) {
        Ok(path) => Transition::with(
            model,
            Effect::Fetch {
                method: Method::Get,
                path,
                query: Vec::new(),
                body: None,
                title: "JWKS".to_owned(),
            },
        ),
        Err(error) => {
            Transition::pure(model.noticed(Notice::from_error("cannot read keys", &error)))
        }
    }
}

/// Offer to write whatever the viewer is showing to a file.
#[must_use]
pub fn save_viewer(model: Model) -> Transition {
    let Some(Overlay::Viewer { title, body, .. }) = &model.overlay else {
        return Transition::pure(model);
    };

    let suggested = format!(
        "{}.json",
        title
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_lowercase()
    );
    let body = Arc::clone(body);

    Transition::pure(model.with_overlay(Overlay::Prompt(crate::model::Prompt {
        title: Arc::from("Save to file"),
        help: Arc::from("Path to write. An existing file is overwritten."),
        input: crate::text_input::TextInput::new(suggested),
        secret: false,
        ask: crate::model::Ask::SaveToFile { body },
    })))
}
