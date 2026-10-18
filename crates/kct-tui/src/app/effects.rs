use std::sync::Arc;

use kct_core::ports::{Gateway, Rest};
use kct_core::{ApiPath, Error, Method};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use crate::effect::Effect;
use crate::msg::{Msg, Notice};

use super::requests::{call, fetch_body, load};
use super::{clipboard, files, scim, tokens};

/// Interpret one effect. Each produces at most one data message plus a log entry, so the
/// update loop stays the only place that decides anything.
pub async fn run(
    effect: Effect,
    gateway: Arc<dyn Gateway>,
    tokens_source: Arc<dyn kct_core::ports::TokenSource>,
    tx: &UnboundedSender<Msg>,
) {
    match effect {
        Effect::Load {
            token,
            fetch,
            path,
            query,
            shape,
        } => load(token, fetch, path, query, shape, &gateway, tx).await,

        Effect::Call {
            method,
            path,
            query,
            body,
            title,
        } => call(method, path, query, body, title, &gateway, tx).await,

        Effect::Fetch {
            method,
            path,
            query,
            body,
            title,
        } => fetch_body(method, path, query, body, title, &gateway, tx).await,

        Effect::InspectToken => tokens::inspect(&tokens_source, tx).await,
        Effect::CopyToken => tokens::copy(&tokens_source, tx).await,

        Effect::CopyCurl {
            method,
            path,
            base_url,
        } => {
            let command = files::curl(method, &base_url, &path);
            let _ = tx.send(Msg::Notice(match clipboard::copy(&command) {
                Ok(()) => Notice::success("curl command copied"),
                Err(reason) => Notice::warning(format!("could not copy: {reason}")),
            }));
        }

        Effect::ClientToken {
            realm,
            client_id,
            secret_path,
        } => tokens::for_client(realm, client_id, secret_path, &gateway, tx).await,

        Effect::ScimProbe {
            realm,
            client_id,
            secret_path,
        } => scim::probe(realm, client_id, secret_path, &gateway, tx).await,

        Effect::WriteFile { path, contents } => files::write(&path, &contents, tx).await,

        Effect::ImportFile {
            source,
            target,
            policy,
        } => files::import(&source, target, policy, &gateway, tx).await,

        Effect::DetectCapabilities { realm } => {
            use kct_core::ports::AdminApi;
            let result = gateway.server_profile(&realm).await;
            log_status(
                tx,
                Method::Get,
                &ApiPath::new("/admin/serverinfo"),
                result.as_ref().map(|_| 200),
            );
            let _ = tx.send(Msg::Profile(result));
        }

        Effect::DetectUserProfile { .. }
        | Effect::DetectRealmFlags { .. }
        | Effect::DetectAuthFlows { .. } => detect(effect, &gateway, tx).await,

        Effect::LoadMascots => {
            let _ = tx.send(Msg::Mascots(crate::view::mascot_art::scan()));
        }

        Effect::ListRealms => {
            use kct_core::ports::AdminApi;
            let _ = tx.send(Msg::Realms(gateway.realms().await.map(Arc::from)));
        }

        Effect::Copy { text, label } => {
            let _ = tx.send(Msg::Notice(match clipboard::copy(&text) {
                Ok(()) => Notice::success(format!("{label} copied")),
                Err(reason) => Notice::warning(format!("could not copy {label}: {reason}")),
            }));
        }

        Effect::Quit => {}
    }
}

pub(super) fn log(
    tx: &UnboundedSender<Msg>,
    method: Method,
    path: &ApiPath,
    result: &Result<Value, Error>,
) {
    log_status(tx, method, path, result.as_ref().map(|_| 200));
}

pub(super) fn log_status(
    tx: &UnboundedSender<Msg>,
    method: Method,
    path: &ApiPath,
    outcome: Result<u16, &Error>,
) {
    let _ = tx.send(Msg::Logged {
        method,
        path: path.as_str().to_owned(),
        outcome: outcome.map_err(|error| {
            error
                .status()
                .map_or_else(|| error.to_string(), |status| status.to_string())
        }),
    });
}

/// The three realm-scoped probes that run once at startup.
///
/// They share a shape — resolve a path, fetch it, log it, send the answer — and a failure on
/// any of them is survivable, so none of them interrupts the operator.
async fn detect(effect: Effect, gateway: &Arc<dyn Gateway>, tx: &UnboundedSender<Msg>) {
    use kct_core::path::{PathParams, endpoints};

    let (template, realm) = match &effect {
        Effect::DetectUserProfile { realm } => (endpoints::USER_PROFILE, realm),
        Effect::DetectRealmFlags { realm } => (endpoints::REALM, realm),
        Effect::DetectAuthFlows { realm } => (endpoints::AUTH_FLOWS, realm),
        _ => return,
    };
    let Ok(path) = template.resolve(&PathParams::realm(realm)) else {
        return;
    };

    let result = gateway.get_json(path.clone(), Vec::new()).await;
    log(tx, Method::Get, &path, &result);
    let _ = tx.send(match effect {
        Effect::DetectUserProfile { .. } => Msg::UserProfile(result),
        Effect::DetectRealmFlags { .. } => Msg::RealmFlags(result),
        _ => Msg::AuthFlows(
            result.map(|body| Arc::from(body.as_array().cloned().unwrap_or_default().as_slice())),
        ),
    });
}
