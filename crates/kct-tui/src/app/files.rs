use std::path::Path;
use std::sync::Arc;

use kct_core::ports::Gateway;
use kct_core::{ApiPath, Method, Request, ports::Expect};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use crate::msg::{Msg, Notice};

/// Write a viewer's contents to disk.
pub(super) async fn write(path: &Path, contents: &str, tx: &UnboundedSender<Msg>) {
    let notice = match tokio::fs::write(path, contents).await {
        Ok(()) => Notice::success(format!(
            "wrote {} bytes to {}",
            contents.len(),
            path.display()
        )),
        Err(error) => Notice::failure(format!("could not write {}: {error}", path.display())),
    };
    let _ = tx.send(Msg::Notice(notice));
}

/// Read a realm export from disk and send it back as a partial import.
///
/// The `ifResourceExists` policy travels in the body alongside the representation; it is
/// stated explicitly here because the default an operator assumes and the one Keycloak uses
/// are not always the same.
pub(super) async fn import(
    source: &Path,
    target: ApiPath,
    policy: &str,
    gateway: &Arc<dyn Gateway>,
    tx: &UnboundedSender<Msg>,
) {
    let text = match tokio::fs::read_to_string(source).await {
        Ok(text) => text,
        Err(error) => {
            let _ = tx.send(Msg::Notice(Notice::failure(format!(
                "could not read {}: {error}",
                source.display()
            ))));
            return;
        }
    };

    let Ok(Value::Object(mut body)) = serde_json::from_str::<Value>(&text) else {
        let _ = tx.send(Msg::Notice(Notice::failure(format!(
            "{} is not a JSON object; a partial import needs a realm representation",
            source.display()
        ))));
        return;
    };
    body.insert(
        "ifResourceExists".to_owned(),
        Value::String(policy.to_owned()),
    );

    let result = gateway
        .send(
            Request::new(Method::Post, target)
                .with_body(Value::Object(body))
                .expecting(Expect::Json),
        )
        .await;

    let _ = tx.send(Msg::Done {
        title: format!("imported {} ({policy})", source.display()),
        result: result.map(|response| {
            response
                .body
                .get("added")
                .and_then(Value::as_u64)
                .map(|added| format!("{added} objects added"))
        }),
    });
}

/// Build a shell command that reproduces one admin call.
#[must_use]
pub(super) fn curl(method: Method, base_url: &str, path: &str) -> String {
    format!(
        "# export KC_TOKEN=... first; the token is deliberately not pasted here\n\
         curl -sS -X {method} '{base_url}{path}' \\\n  \
         -H \"Authorization: Bearer $KC_TOKEN\" \\\n  \
         -H 'Accept: application/json'\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reproducible_command_never_embeds_the_bearer() {
        let command = curl(
            Method::Get,
            "https://kc.example.com",
            "/admin/realms/avsystem/users",
        );

        assert!(command.contains("$KC_TOKEN"));
        assert!(command.contains("https://kc.example.com/admin/realms/avsystem/users"));
        assert!(command.contains("-X GET"));
    }
}
