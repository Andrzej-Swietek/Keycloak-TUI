use super::prelude::*;

pub(crate) const REALM_GENERAL_FIELDS: &[Field] = &[
    Field::new("Realm", "/realm", FieldKind::ReadOnly),
    Field::new("Display name", "/displayName", FieldKind::Text),
    Field::new("HTML display name", "/displayNameHtml", FieldKind::Text),
    Field::new("Enabled", "/enabled", FieldKind::Bool),
    Field::new(
        "SSL required",
        "/sslRequired",
        FieldKind::Choice(&["all", "external", "none"]),
    )
    .help("`none` must never be used outside a throwaway lab."),
    Field::new(
        "User-managed access",
        "/userManagedAccessAllowed",
        FieldKind::Bool,
    ),
    Field::new(
        "Organizations enabled",
        "/organizationsEnabled",
        FieldKind::Bool,
    )
    .needs(KC_26),
    Field::new(
        "Admin permissions (FGAP v2)",
        "/adminPermissionsEnabled",
        FieldKind::Bool,
    )
    .help(
        "Switching this on creates a client named `admin-permissions`; the grants themselves \
         are its authorization permissions.",
    )
    .needs(KC_26_2),
    Field::new(
        "Default signature algorithm",
        "/defaultSignatureAlgorithm",
        FieldKind::Text,
    ),
    Field::new("Attributes", "/attributes", FieldKind::StringMap),
];

pub(crate) const REALM_LOGIN_FIELDS: &[Field] = &[
    Field::new("User registration", "/registrationAllowed", FieldKind::Bool),
    Field::new(
        "Email as username",
        "/registrationEmailAsUsername",
        FieldKind::Bool,
    ),
    Field::new("Edit username", "/editUsernameAllowed", FieldKind::Bool),
    Field::new("Forgot password", "/resetPasswordAllowed", FieldKind::Bool),
    Field::new("Remember me", "/rememberMe", FieldKind::Bool),
    Field::new("Verify email", "/verifyEmail", FieldKind::Bool),
    Field::new(
        "Login with email",
        "/loginWithEmailAllowed",
        FieldKind::Bool,
    ),
    Field::new(
        "Duplicate emails",
        "/duplicateEmailsAllowed",
        FieldKind::Bool,
    ),
    Field::new("Password policy", "/passwordPolicy", FieldKind::Text)
        .help("e.g. length(12) and upperCase(1) and notUsername() and passwordHistory(3)"),
];

pub(crate) const REALM_SECURITY_FIELDS: &[Field] = &[
    Field::new(
        "Brute force protected",
        "/bruteForceProtected",
        FieldKind::Bool,
    ),
    Field::new("Permanent lockout", "/permanentLockout", FieldKind::Bool),
    Field::new("Max login failures", "/failureFactor", FieldKind::Integer),
    Field::new(
        "Wait increment (s)",
        "/waitIncrementSeconds",
        FieldKind::Seconds,
    ),
    Field::new("Max wait (s)", "/maxFailureWaitSeconds", FieldKind::Seconds),
    Field::new(
        "Failure reset (s)",
        "/maxDeltaTimeSeconds",
        FieldKind::Seconds,
    ),
    Field::new(
        "Quick login check (ms)",
        "/quickLoginCheckMilliSeconds",
        FieldKind::Integer,
    ),
    Field::new(
        "Min quick login wait (s)",
        "/minimumQuickLoginWaitSeconds",
        FieldKind::Seconds,
    ),
    Field::new("Default role", "/defaultRole/name", FieldKind::ReadOnly)
        .help("Roles every new user gets. Change its composites on the Realm roles screen."),
    Field::new(
        "Required credentials",
        "/requiredCredentials",
        FieldKind::List,
    ),
    Field::new("SCIM API", "/scimApiEnabled", FieldKind::Bool)
        .help("Serves SCIM 2.0 under /realms/{realm}/scim/v2. Supported and on by default from 26.8; preview behind `scim-api` before that.")
        .needs(Requirement::Feature(features::SCIM_API)),
    Field::new(
        "Verifiable credentials",
        "/verifiableCredentialsEnabled",
        FieldKind::Bool,
    ),
    Field::new(
        "Brute force strategy",
        "/bruteForceStrategy",
        FieldKind::Choice(&["LINEAR", "MULTIPLE"]),
    )
    .help("`MULTIPLE` lengthens each temporary lockout; `LINEAR` keeps them the same."),
    Field::new(
        "Max temporary lockouts",
        "/maxTemporaryLockouts",
        FieldKind::Integer,
    )
    .help("Temporary lockouts before the account is disabled permanently. 0 never disables."),
    Field::new(
        "Max secondary auth failures",
        "/maxSecondaryAuthFailures",
        FieldKind::Integer,
    )
    .help("Failures of a second factor counted separately from the password."),
    Field::new(
        "Security headers",
        "/browserSecurityHeaders",
        FieldKind::StringMap,
    )
    .help("Headers Keycloak adds to its own pages: CSP, X-Frame-Options, HSTS."),
];
