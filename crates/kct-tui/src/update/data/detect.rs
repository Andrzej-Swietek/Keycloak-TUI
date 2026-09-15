//! What the session learns about the server and the realm, once, in the background.
//!
//! Every one of these can fail without stopping anyone: a realm-scoped admin cannot list
//! realms, an old server has no user profile. So a failure is recorded as "unknown" rather
//! than raised, and the screens that care gate themselves on what is known.

use std::sync::Arc;

use kct_core::Error;
use kct_core::capability::ServerProfile;
use serde_json::Value;

use crate::model::Model;
use crate::msg::Notice;

use super::super::{nav, transition::Transition};

pub fn profile(model: Model, result: Result<ServerProfile, Error>) -> Transition {
    match result {
        Ok(profile) => {
            let warnings = profile
                .risky_enabled_features()
                .iter()
                .map(|flag| format!("{} ({})", flag.name, flag.kind.badge()))
                .collect::<Vec<_>>();

            let model = Model {
                session: crate::model::Session {
                    profile,
                    ..model.session
                },
                ..model
            };

            if warnings.is_empty() {
                Transition::pure(model)
            } else {
                Transition::pure(model.noticed(Notice::warning(format!(
                    "not-GA features in use: {}",
                    warnings.join(", ")
                ))))
            }
        }
        Err(error) => Transition::pure(
            model.noticed(Notice::from_error("capability detection failed", &error)),
        ),
    }
}

pub fn realms(model: Model, result: Result<Arc<[String]>, Error>) -> Transition {
    match result {
        Ok(realms) => Transition::pure(Model {
            session: crate::model::Session {
                realms,
                ..model.session
            },
            ..model
        }),
        // Listing realms needs master-realm rights that a realm-scoped admin will not have;
        // that is normal, not an error worth interrupting anyone over.
        Err(_) => Transition::pure(model),
    }
}

/// Record what the realm does with undeclared attributes.
pub fn user_profile(model: Model, result: Result<Value, Error>) -> Transition {
    let Ok(profile) = result else {
        return Transition::pure(model);
    };

    Transition::pure(Model {
        session: crate::model::Session {
            unmanaged_attributes: crate::model::UnmanagedAttributes::from_user_profile(&profile),
            declared_attributes: Arc::from(crate::model::UnmanagedAttributes::declared(&profile)),
            ..model.session
        },
        ..model
    })
}

/// Take a fresh scan of the mascot directory, reporting anything it could not use.
pub fn mascots(model: Model, scan: crate::view::Scan) -> Transition {
    let count = scan.found.len();
    let problems = scan.problems.clone();
    let model = Model {
        prefs: model.prefs.clamped(count),
        mascots: scan,
        ..model
    };

    let said = match count {
        0 => "no custom mascots found".to_owned(),
        1 => "loaded 1 custom mascot".to_owned(),
        many => format!("loaded {many} custom mascots"),
    };
    let model = problems
        .iter()
        .fold(model.noticed(Notice::info(said)), |model, problem| {
            model.noticed(Notice::warning(problem.clone()))
        });
    Transition::pure(model)
}

/// Remember the realm's top-level flows, so binding a flow offers what exists.
pub fn auth_flows(model: Model, result: Result<Arc<[Value]>, Error>) -> Transition {
    let Ok(flows) = result else {
        return Transition::pure(model);
    };

    let aliases: Vec<String> = flows
        .iter()
        .filter(|flow| flow.get("topLevel").and_then(Value::as_bool) != Some(false))
        .filter_map(|flow| flow.get("alias").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect();

    Transition::pure(Model {
        session: crate::model::Session {
            auth_flows: Arc::from(aliases),
            ..model.session
        },
        ..model
    })
}

/// Record the realm's optional-feature opt-ins, then re-gate the current screen.
///
/// A feature GA on the server but off in the realm answers 404, not 403.
pub fn realm_flags(model: Model, result: Result<Value, Error>) -> Transition {
    let Ok(realm) = result else {
        return Transition::pure(model);
    };

    let model = Model {
        session: crate::model::Session {
            realm_flags: Arc::new(realm),
            ..model.session
        },
        ..model
    };

    // The flags arrive after the first fetch, so a screen the realm has switched off has
    // already produced a bare 404 by now. Re-run the gate to replace it with the reason.
    if model.session.support_for(model.resource()).is_permitted() {
        Transition::pure(model)
    } else {
        nav::reload(model)
    }
}
