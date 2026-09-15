/// What choosing a palette entry does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Navigate to a resource listing.
    Open(&'static str),
    /// Run an action of the resource currently in view.
    Run {
        resource: &'static str,
        action: &'static str,
    },
    Realm(String),
    Global(Global),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Global {
    Help,
    Refresh,
    Back,
    Quit,
    SwitchRealm,
    ReAuthenticate,
    CopyAccessToken,
    InspectToken,
    RealmJwks,
    CopyAsCurl,
    ToggleDryRun,
    ToggleMascot,
    ReloadMascots,
    ServerInfo,
    RequestLog,
}

impl Global {
    pub(super) const ALL: &'static [(Self, &'static str, &'static str, &'static str)] = &[
        (
            Self::SwitchRealm,
            "Switch realm",
            "Pick another realm to administer",
            "change tenant select",
        ),
        (
            Self::Refresh,
            "Refresh",
            "Re-fetch the current view",
            "reload update",
        ),
        (Self::Back, "Back", "Leave this view", "up parent return"),
        (
            Self::ServerInfo,
            "Server info",
            "Version, SPIs and feature flags",
            "version diagnostics spi",
        ),
        (
            Self::RequestLog,
            "Request log",
            "Every admin API call this session made",
            "debug http trace audit",
        ),
        (
            Self::InspectToken,
            "Inspect my access token",
            "Decode it: scope, audience, roles and expiry",
            "jwt claims decode debug why 403 forbidden",
        ),
        (
            Self::CopyAccessToken,
            "Copy access token",
            "Put the bearer token on the clipboard",
            "jwt bearer curl",
        ),
        (
            Self::RealmJwks,
            "Show realm JWKS",
            "The public keys a resource server validates signatures with",
            "certs keys jwks signature rotation",
        ),
        (
            Self::CopyAsCurl,
            "Copy last request as curl",
            "Reproduce the last call outside the tool",
            "debug reproduce shell",
        ),
        (
            Self::ToggleMascot,
            "Change the mascot",
            "Cycle through the characters, or hide the corner entirely",
            "mascot anime character twintails cat robot ghost hide corner",
        ),
        (
            Self::ReloadMascots,
            "Reload mascot art",
            "Rescan the mascot directory, so art you just drew appears without a restart",
            "mascot reload rescan custom art png image sprite drop-in",
        ),
        (
            Self::ToggleDryRun,
            "Toggle dry-run",
            "Suppress every write and report what would be sent",
            "safe readonly rehearse",
        ),
        (
            Self::ReAuthenticate,
            "Re-authenticate",
            "Obtain a fresh access token",
            "login relogin expired",
        ),
        (Self::Help, "Help", "Keyboard shortcuts", "keys shortcuts ?"),
        (Self::Quit, "Quit", "Leave kct", "exit close"),
    ];
}
