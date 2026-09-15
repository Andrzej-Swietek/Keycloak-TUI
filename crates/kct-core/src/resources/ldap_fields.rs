use super::prelude::*;

pub(crate) const LDAP_CONNECTION_FIELDS: &[Field] = &[
    Field::new("ID", "/id", FieldKind::ReadOnly),
    Field::new("Name", "/name", FieldKind::Text),
    Field::new("Provider", "/providerId", FieldKind::ReadOnly),
    Field::config("Enabled", "/config/enabled", FieldKind::Bool),
    Field::config(
        "Vendor",
        "/config/vendor",
        FieldKind::Choice(&["ad", "rhds", "tivoli", "edirectory", "other"]),
    ),
    Field::config("Connection URL", "/config/connectionUrl", FieldKind::Text)
        .help("Use ldaps:// in production; the server truststore must trust the directory's CA."),
    Field::config("Enable StartTLS", "/config/startTls", FieldKind::Bool),
    Field::config(
        "Use truststore SPI",
        "/config/useTruststoreSpi",
        FieldKind::Choice(&["always", "never", "ldapsOnly"]),
    ),
    Field::config(
        "Bind type",
        "/config/authType",
        FieldKind::Choice(&["simple", "none"]),
    ),
    Field::config("Bind DN", "/config/bindDn", FieldKind::Text),
    Field::config(
        "Bind credential",
        "/config/bindCredential",
        FieldKind::Secret,
    ),
    Field::config(
        "Connection pooling",
        "/config/connectionPooling",
        FieldKind::Bool,
    ),
    Field::config(
        "Connection timeout (ms)",
        "/config/connectionTimeout",
        FieldKind::Integer,
    ),
    Field::config(
        "Read timeout (ms)",
        "/config/readTimeout",
        FieldKind::Integer,
    ),
    Field::config(
        "Connection trace",
        "/config/connectionTrace",
        FieldKind::Bool,
    )
    .help("Logs the JNDI conversation. Verbose; for diagnosing a bind that will not work."),
];

pub(crate) const LDAP_SEARCH_FIELDS: &[Field] = &[
    Field::config("Users DN", "/config/usersDn", FieldKind::Text),
    Field::config(
        "Username LDAP attribute",
        "/config/usernameLDAPAttribute",
        FieldKind::Text,
    )
    .help("`uid` for most directories, `cn` or `sAMAccountName` for Active Directory."),
    Field::config(
        "RDN LDAP attribute",
        "/config/rdnLDAPAttribute",
        FieldKind::Text,
    ),
    Field::config(
        "UUID LDAP attribute",
        "/config/uuidLDAPAttribute",
        FieldKind::Text,
    )
    .help("`entryUUID` for most directories, `objectGUID` for Active Directory."),
    Field::config(
        "User object classes",
        "/config/userObjectClasses",
        FieldKind::Text,
    ),
    Field::config(
        "User LDAP filter",
        "/config/customUserSearchFilter",
        FieldKind::Text,
    )
    .help("Must be a valid LDAP filter enclosed in parentheses."),
    Field::config(
        "Search scope",
        "/config/searchScope",
        FieldKind::Choice(&["1", "2"]),
    )
    .help("1 = one level, 2 = subtree."),
    Field::config(
        "Referral",
        "/config/referral",
        FieldKind::Choice(&["ignore", "follow"]),
    ),
    Field::config("Pagination", "/config/pagination", FieldKind::Bool),
    Field::config(
        "Relative create DN",
        "/config/relativeCreateDn",
        FieldKind::Text,
    )
    .help("Sub-tree, relative to Users DN, that new users are created in."),
];

pub(crate) const LDAP_SYNC_FIELDS: &[Field] = &[
    Field::config("Import users", "/config/importEnabled", FieldKind::Bool),
    Field::config(
        "Edit mode",
        "/config/editMode",
        FieldKind::Choice(&["READ_ONLY", "WRITABLE", "UNSYNCED"]),
    )
    .help("WRITABLE propagates changes back to LDAP; UNSYNCED keeps them local only."),
    Field::config(
        "Sync registrations",
        "/config/syncRegistrations",
        FieldKind::Bool,
    ),
    Field::config("Batch size", "/config/batchSizeForSync", FieldKind::Integer),
    Field::config(
        "Full sync period (s)",
        "/config/fullSyncPeriod",
        FieldKind::Seconds,
    ),
    Field::config(
        "Changed sync period (s)",
        "/config/changedSyncPeriod",
        FieldKind::Seconds,
    ),
    Field::config(
        "Validate password policy",
        "/config/validatePasswordPolicy",
        FieldKind::Bool,
    ),
    Field::config("Trust email", "/config/trustEmail", FieldKind::Bool),
    Field::config(
        "Cache policy",
        "/config/cachePolicy",
        FieldKind::Choice(&[
            "DEFAULT",
            "EVICT_DAILY",
            "EVICT_WEEKLY",
            "MAX_LIFESPAN",
            "NO_CACHE",
        ]),
    ),
    Field::config(
        "Use LDAP password policy",
        "/config/enableLdapPasswordPolicy",
        FieldKind::Bool,
    )
    .help("Let the directory enforce its own policy and surface its rejection."),
    Field::config(
        "Password modify extended op",
        "/config/usePasswordModifyExtendedOp",
        FieldKind::Bool,
    )
    .help("Use the RFC 3062 extended operation instead of writing userPassword directly."),
];

pub(crate) const LDAP_KERBEROS_FIELDS: &[Field] = &[
    Field::config(
        "Allow Kerberos authentication",
        "/config/allowKerberosAuthentication",
        FieldKind::Bool,
    ),
    Field::config("Kerberos realm", "/config/kerberosRealm", FieldKind::Text),
    Field::config(
        "Server principal",
        "/config/serverPrincipal",
        FieldKind::Text,
    ),
    Field::config("Key tab", "/config/keyTab", FieldKind::Text),
    Field::config(
        "Kerberos principal attribute",
        "/config/krbPrincipalAttribute",
        FieldKind::Text,
    ),
    Field::config("Debug", "/config/debug", FieldKind::Bool),
    Field::config(
        "Use Kerberos for password auth",
        "/config/useKerberosForPasswordAuthentication",
        FieldKind::Bool,
    ),
    Field::config(
        "Kerberos principal attribute",
        "/config/krbPrincipalAttribute",
        FieldKind::Text,
    ),
];
