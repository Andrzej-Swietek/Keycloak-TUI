use anyhow::{Context as _, Result, bail};
use kct_client::{Connection, Credentials, TlsOptions};
use kct_core::Secret;

use crate::cli::Cli;
use crate::config::ContextConfig;
use crate::prompt;

/// Merge the command line over the saved context and build a connection.
///
/// Anything the operator typed wins; the context fills the rest. Credentials are resolved
/// last so the choice of grant is explicit in one place.
pub fn build(cli: &Cli, context: &ContextConfig) -> Result<Connection> {
    let server = cli
        .server
        .clone()
        .or_else(|| context.server.clone())
        .context(
            "no server URL. Pass --server, set KCT_SERVER, or define a context in the config file",
        )?;

    let realm = pick(&cli.realm, &context.realm, "master");
    let auth_realm = cli
        .auth_realm
        .clone()
        .or_else(|| context.auth_realm.clone())
        .unwrap_or_else(|| "master".to_owned());

    let tls = TlsOptions {
        insecure: cli.insecure || context.insecure,
        ca_bundle: cli
            .ca_cert
            .clone()
            .or_else(|| context.ca_cert.clone())
            .map(|path| std::fs::read(&path).with_context(|| format!("reading {}", path.display())))
            .transpose()?,
    };

    Connection::new(server, auth_realm, realm, credentials(cli, context)?)
        .map_err(|error| anyhow::anyhow!("{error}"))
        .map(|connection| connection.with_tls(tls).with_dry_run(cli.dry_run))
}

fn credentials(cli: &Cli, context: &ContextConfig) -> Result<Credentials> {
    let client_id = pick(&cli.client_id, &context.client_id, "admin-cli");
    let client_secret = cli
        .client_secret
        .clone()
        .or_else(|| context.client_secret.clone());
    let username = cli.user.clone().or_else(|| context.username.clone());

    match (&cli.token, username, &client_secret) {
        (Some(token), _, _) => Ok(Credentials::Bearer(Secret::new(token.clone()))),

        (None, Some(username), _) => {
            let password = match &cli.password {
                Some(password) => password.clone(),
                None => prompt::password(&format!("password for {username}"))?,
            };
            Ok(Credentials::Password {
                client_id,
                username,
                password: Secret::new(password),
                client_secret: client_secret.map(Secret::new),
                otp: cli.otp.clone().map(Secret::new),
            })
        }

        (None, None, Some(secret)) => Ok(Credentials::ClientCredentials {
            client_id,
            client_secret: Secret::new(secret.clone()),
        }),

        (None, None, None) => bail!(
            "no credentials. Pass --user (password grant), --client-secret (service account) \
             or --token"
        ),
    }
}

fn pick(cli: &Option<String>, context: &Option<String>, fallback: &str) -> String {
    cli.clone()
        .or_else(|| context.clone())
        .unwrap_or_else(|| fallback.to_owned())
}
