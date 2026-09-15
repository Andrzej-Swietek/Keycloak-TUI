use super::prelude::*;

pub(crate) const REALM_SESSION_FIELDS: &[Field] = &[
    Field::new(
        "Client offline idle (s)",
        "/clientOfflineSessionIdleTimeout",
        FieldKind::Seconds,
    )
    .help("Overrides the realm offline idle timeout for clients that set their own. 0 defers."),
    Field::new(
        "Client offline max (s)",
        "/clientOfflineSessionMaxLifespan",
        FieldKind::Seconds,
    ),
    Field::new(
        "SSO session idle",
        "/ssoSessionIdleTimeout",
        FieldKind::Seconds,
    ),
    Field::new(
        "SSO session max",
        "/ssoSessionMaxLifespan",
        FieldKind::Seconds,
    ),
    Field::new(
        "SSO idle (remember me)",
        "/ssoSessionIdleTimeoutRememberMe",
        FieldKind::Seconds,
    ),
    Field::new(
        "SSO max (remember me)",
        "/ssoSessionMaxLifespanRememberMe",
        FieldKind::Seconds,
    ),
    Field::new(
        "Client session idle",
        "/clientSessionIdleTimeout",
        FieldKind::Seconds,
    ),
    Field::new(
        "Client session max",
        "/clientSessionMaxLifespan",
        FieldKind::Seconds,
    ),
    Field::new(
        "Offline session idle",
        "/offlineSessionIdleTimeout",
        FieldKind::Seconds,
    ),
    Field::new(
        "Offline max enabled",
        "/offlineSessionMaxLifespanEnabled",
        FieldKind::Bool,
    ),
    Field::new(
        "Offline session max",
        "/offlineSessionMaxLifespan",
        FieldKind::Seconds,
    ),
];

pub(crate) const REALM_TOKEN_FIELDS: &[Field] = &[
    Field::new(
        "Access token lifespan",
        "/accessTokenLifespan",
        FieldKind::Seconds,
    ),
    Field::new(
        "Access token (implicit)",
        "/accessTokenLifespanForImplicitFlow",
        FieldKind::Seconds,
    ),
    Field::new(
        "Client login timeout",
        "/accessCodeLifespan",
        FieldKind::Seconds,
    ),
    Field::new(
        "Login action timeout",
        "/accessCodeLifespanUserAction",
        FieldKind::Seconds,
    ),
    Field::new(
        "Login timeout",
        "/accessCodeLifespanLogin",
        FieldKind::Seconds,
    ),
    Field::new(
        "Admin action token lifespan",
        "/actionTokenGeneratedByAdminLifespan",
        FieldKind::Seconds,
    ),
    Field::new(
        "User action token lifespan",
        "/actionTokenGeneratedByUserLifespan",
        FieldKind::Seconds,
    ),
    Field::new(
        "Revoke refresh token",
        "/revokeRefreshToken",
        FieldKind::Bool,
    ),
    Field::new(
        "Refresh token max reuse",
        "/refreshTokenMaxReuse",
        FieldKind::Integer,
    ),
];

pub(crate) const REALM_THEME_FIELDS: &[Field] = &[
    Field::new("Login theme", "/loginTheme", FieldKind::Text),
    Field::new("Account theme", "/accountTheme", FieldKind::Text),
    Field::new("Admin theme", "/adminTheme", FieldKind::Text),
    Field::new("Email theme", "/emailTheme", FieldKind::Text),
    Field::new(
        "Internationalization",
        "/internationalizationEnabled",
        FieldKind::Bool,
    ),
    Field::new("Supported locales", "/supportedLocales", FieldKind::List),
    Field::new("Default locale", "/defaultLocale", FieldKind::Text),
];

pub(crate) const REALM_EMAIL_FIELDS: &[Field] = &[
    Field::new("From", "/smtpServer/from", FieldKind::Text),
    Field::new(
        "From display name",
        "/smtpServer/fromDisplayName",
        FieldKind::Text,
    ),
    Field::new("Reply-to", "/smtpServer/replyTo", FieldKind::Text),
    Field::new("Host", "/smtpServer/host", FieldKind::Text),
    Field::new("Port", "/smtpServer/port", FieldKind::Text),
    Field::stringly("SSL", "/smtpServer/ssl", FieldKind::Bool),
    Field::stringly("StartTLS", "/smtpServer/starttls", FieldKind::Bool),
    Field::stringly("Authentication", "/smtpServer/auth", FieldKind::Bool),
    Field::new("Username", "/smtpServer/user", FieldKind::Text),
    Field::new("Password", "/smtpServer/password", FieldKind::Secret),
];
