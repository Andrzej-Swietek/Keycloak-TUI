//! Entry point: parse the command line, build a client, hand control to the UI.

mod cli;
mod config;
mod connect;
mod doctor;
mod mascot;
mod prompt;

use std::sync::Arc;

use anyhow::{Context as _, Result};
use clap::Parser;
use kct_client::KeycloakClient;
use kct_core::resources;
use kct_tui::model::Session;
use kct_tui::view::mascot_art;

use mascot::choose_mascot;

use cli::{Cli, Command};
use config::Config;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Command::Doctor { verbose }) = &cli.command {
        return check(&cli, *verbose);
    }
    if let Some(command) = &cli.command {
        return report(command);
    }

    init_logging(cli.log.as_deref())?;

    let config = Config::load()?;
    let context = config.select(cli.context.as_deref())?;
    let connection = connect::build(&cli, &context)?;

    let screen = cli
        .screen
        .clone()
        .or_else(|| context.screen.clone())
        .unwrap_or_else(|| "users".to_owned());
    let start = resources::by_id(&screen)
        .with_context(|| format!("no screen called '{screen}'; try `kct screens`"))?;

    let mascots = mascot_art::scan();
    let mascot = choose_mascot(&cli.mascot, &mascots)?;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(start_ui(connection, start, mascot, mascots))
}

async fn start_ui(
    connection: kct_client::Connection,
    start: &'static kct_core::catalog::Resource,
    mascot: Option<kct_tui::model::Mascot>,
    mascots: kct_tui::view::Scan,
) -> Result<()> {
    let session = Session {
        prefs: kct_tui::model::Prefs { mascot },
        dry_run: connection.dry_run,
        insecure: connection.tls.insecure,
        cleartext: connection.cleartext_warning().map(Arc::from),
        ..Session::new(
            connection.base_url.clone(),
            connection.realm.clone(),
            connection.credentials.describe(),
        )
    };

    let client = KeycloakClient::connect(connection).map_err(|error| anyhow::anyhow!("{error}"))?;

    // Authenticate before taking over the terminal: a wrong password should be a plain
    // message on the shell, not a modal inside a UI that then has nothing to show.
    client
        .authenticate()
        .await
        .map_err(|error| match error.hint() {
            Some(hint) => anyhow::anyhow!("{error}\n{hint}"),
            None => anyhow::anyhow!("{error}"),
        })?;

    // The same object serves both ports: the admin API, and read access to the token the
    // session is presenting.
    let client = Arc::new(client);
    kct_tui::app::run(
        session,
        client.clone() as Arc<dyn kct_core::Gateway>,
        client as Arc<dyn kct_core::TokenSource>,
        start,
        mascots,
    )
    .await
    .map_err(Into::into)
}

/// `kct doctor` needs a server, so it goes through the same connection path as the UI.
fn check(cli: &Cli, verbose: bool) -> Result<()> {
    let config = Config::load()?;
    let context = config.select(cli.context.as_deref())?;
    let connection = connect::build(cli, &context)?;
    let realm = connection.realm.clone();

    let client = KeycloakClient::connect(connection).map_err(|error| anyhow::anyhow!("{error}"))?;

    let report = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            client
                .authenticate()
                .await
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            doctor::run(&client, &realm).await
        })?;

    doctor::print(&report, verbose);

    if report.is_healthy() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn report(command: &Command) -> Result<()> {
    match command {
        Command::Contexts => {
            let config = Config::load()?;
            for name in config.names() {
                let marker = if config.default.as_deref() == Some(name) {
                    "*"
                } else {
                    " "
                };
                println!("{marker} {name}");
            }
        }
        Command::ConfigPath => match Config::path() {
            Some(path) => println!("{}", path.display()),
            None => println!("(no config directory on this platform)"),
        },
        Command::Doctor { .. } => unreachable!("handled before the runtime is built"),
        Command::Mascots { grid } => mascot::show(*grid),
        Command::Screens => {
            for resource in resources::top_level() {
                println!("{:<24} {}", resource.id, resource.summary);
            }
        }
    }
    Ok(())
}

/// A TUI cannot log to the terminal it is drawing on, so logging goes to a file or nowhere.
fn init_logging(path: Option<&std::path::Path>) -> Result<()> {
    let Some(path) = path else { return Ok(()) };

    let file = std::fs::File::create(path)
        .with_context(|| format!("creating log file {}", path.display()))?;

    tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(file))
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("KCT_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("kct=debug,info")),
        )
        .with_ansi(false)
        .init();

    Ok(())
}
