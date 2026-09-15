use clap::{Parser, Subcommand};

/// A terminal UI for administering Keycloak.
#[derive(Debug, Parser)]
#[command(name = "kct", version, about, long_about = None)]
pub struct Cli {
    /// Keycloak base URL, without a trailing slash or `/admin`.
    #[arg(short, long, env = "KCT_SERVER")]
    pub server: Option<String>,

    /// Realm to administer.
    #[arg(short, long, env = "KCT_REALM")]
    pub realm: Option<String>,

    /// Realm holding the admin account, if different from the one being administered.
    #[arg(long, env = "KCT_AUTH_REALM")]
    pub auth_realm: Option<String>,

    /// Admin username; enables the password grant.
    #[arg(short, long, env = "KCT_USERNAME")]
    pub user: Option<String>,

    /// Admin password. Prefer the environment variable or the interactive prompt over typing
    /// this on a command line, where it lands in the shell history.
    #[arg(short, long, env = "KCT_PASSWORD", hide_env_values = true)]
    pub password: Option<String>,

    /// OAuth client used to authenticate.
    #[arg(long, env = "KCT_CLIENT_ID")]
    pub client_id: Option<String>,

    /// Client secret. On its own it selects the client-credentials grant.
    #[arg(long, env = "KCT_CLIENT_SECRET", hide_env_values = true)]
    pub client_secret: Option<String>,

    /// A bearer token obtained elsewhere. Cannot be refreshed.
    #[arg(long, env = "KCT_TOKEN", hide_env_values = true)]
    pub token: Option<String>,

    /// One-time code, when the direct-grant flow asks for OTP.
    #[arg(long)]
    pub otp: Option<String>,

    /// Named context from the configuration file.
    #[arg(short, long, env = "KCT_CONTEXT")]
    pub context: Option<String>,

    /// Skip TLS verification. For lab servers only; the session is flagged in the UI.
    #[arg(long)]
    pub insecure: bool,

    /// Extra PEM bundle to trust, in addition to the system trust store.
    #[arg(long, value_name = "PATH")]
    pub ca_cert: Option<std::path::PathBuf>,

    /// Refuse every write and report what would have been sent.
    #[arg(long)]
    pub dry_run: bool,

    /// Who sits in the corner: twintails, miku, cat, robot, ghost, none, or the name of
    /// your own art (see `kct mascots` for where to put it).
    #[arg(long, value_name = "NAME", default_value = "twintails")]
    pub mascot: String,

    /// Screen to open on, by catalog id (`users`, `clients`, `user-federation`, …).
    #[arg(long, value_name = "ID")]
    pub screen: Option<String>,

    /// Write a debug log here. A TUI cannot log to the terminal it is drawing on.
    #[arg(long, value_name = "PATH")]
    pub log: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List the contexts defined in the configuration file.
    Contexts,
    /// Print the path of the configuration file.
    ConfigPath,
    /// List every screen this build knows about.
    Screens,
    /// Print every mascot in every mood, for judging the art without launching the UI.
    Mascots {
        /// Print the raw pixel grid instead, one character per palette key.
        #[arg(short, long)]
        grid: bool,
    },
    /// Check the catalog against a live server and report any drift.
    ///
    /// Exits non-zero when a screen is broken on this server.
    Doctor {
        /// Include the screens that were skipped and why.
        #[arg(short, long)]
        verbose: bool,
    },
}
