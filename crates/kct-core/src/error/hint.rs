use super::{AuthError, Error};

/// Operator-facing remediation. Keycloak's error bodies are terse; this closes the gap.
#[must_use]
pub fn for_error(error: &Error) -> Option<&'static str> {
    match error {
        Error::Api { status: 401, .. } => {
            Some("Session expired or the token was rejected. Re-authenticate to continue.")
        }
        Error::Api {
            status: 403, path, ..
        } => Some(missing_role(path)),
        // A policy type carrying its own required field — a group policy's `groups`, a
        // client policy's `clients` — answers 500 to a create that only names it.
        Error::Api {
            status: 500, path, ..
        } if path.contains("/authz/resource-server/policy/") => Some(
            "This policy type needs its own field set before it will exist. Create one of \
             another type, or send the full representation from the JSON tab.",
        ),
        // Keycloak answers with an opaque 500 when a user-federation provider it has to
        // consult is unreachable, including on writes that look unrelated to it.
        Error::Api { status: 500, .. } => Some(
            "Keycloak hid the cause. A failing user-federation provider is the usual culprit: \
             any user lookup consults it, and an unreachable directory surfaces here rather \
             than on the federation screen. Check User federation, or the server log.",
        ),
        // Keycloak answers "not enabled for this realm" with a 404, which reads like a
        // missing endpoint when it is really an unticked box.
        Error::Api {
            status: 404,
            message,
            detail,
            ..
        } if message.contains("not enabled")
            || detail
                .as_deref()
                .is_some_and(|body| body.contains("not enabled")) =>
        {
            Some(
                "The server supports this, but the realm has not opted in. Realm settings ▸ \
                 General has the switch.",
            )
        }
        Error::Api { status: 404, .. } => Some(
            "Either the object is gone, or this endpoint does not exist on this Keycloak \
             version — check the version in the status bar.",
        ),
        Error::Api { status: 409, .. } => {
            Some("An object with that name already exists in this realm.")
        }
        Error::Api { status: 400, .. } => {
            Some("Keycloak rejected the payload; the JSON tab shows exactly what was sent.")
        }
        Error::Transport { .. } => Some(
            "Check the server URL, DNS, proxy settings and TLS trust (`--insecure` skips \
             verification for lab servers).",
        ),
        Error::Auth(AuthError::InvalidCredentials { .. }) => Some(
            "Verify the client id, whether the client is public or confidential, and that \
             direct access grants are enabled on it. \"Account is not fully set up\" means the \
             account still owes a required action — often a user-profile attribute.",
        ),
        _ => None,
    }
}

/// Name the `realm-management` role a 403 is most likely missing.
///
/// Verified against Keycloak 26.7.3. A delegated admin typically holds a handful of these,
/// so "403" on its own leaves an operator guessing; the role name does not.
fn missing_role(path: &str) -> &'static str {
    const CHECKS: &[(&str, &str)] = &[
        (
            "/authz/",
            "this needs the `view-authorization` (or `manage-authorization`) role.",
        ),
        (
            "/clients",
            "this needs the `view-clients` or `query-clients` role.",
        ),
        ("/client-scopes", "this needs the `view-clients` role."),
        (
            "/users",
            "this needs the `view-users` or `query-users` role.",
        ),
        (
            "/groups",
            "this needs the `view-users` or `query-groups` role.",
        ),
        (
            "/organizations",
            "this needs the `view-organizations` role.",
        ),
        (
            "/identity-provider",
            "this needs the `view-identity-providers` role.",
        ),
        ("/events", "this needs the `view-events` role."),
        ("/components", "this needs the `view-realm` role."),
        ("/authentication", "this needs the `view-realm` role."),
        (
            "/impersonation",
            "this needs the `impersonation` role, and the feature must be enabled.",
        ),
    ];

    match CHECKS.iter().find(|(fragment, _)| path.contains(fragment)) {
        Some((_, advice)) => advice,
        None => {
            "The account lacks the `realm-management` role for this resource, or an FGAP v2 \
             permission denies it."
        }
    }
}
