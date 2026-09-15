use super::prelude::*;

pub(crate) const USER_FIELDS: &[Field] = &[
    Field::new("ID", "/id", FieldKind::ReadOnly),
    Field::new("Username", "/username", FieldKind::Text),
    Field::new("Email", "/email", FieldKind::Text),
    Field::new("Email verified", "/emailVerified", FieldKind::Bool)
        .help("Unverified e-mail blocks account linking on first broker login."),
    Field::new("First name", "/firstName", FieldKind::Text),
    Field::new("Last name", "/lastName", FieldKind::Text),
    Field::new("Enabled", "/enabled", FieldKind::Bool),
    Field::new("Required actions", "/requiredActions", FieldKind::List).help(
        "Actions the user must complete at next login, e.g. UPDATE_PASSWORD, CONFIGURE_TOTP.",
    ),
    Field::new("Federation link", "/federationLink", FieldKind::ReadOnly)
        .help("Set when the user comes from LDAP/Kerberos rather than the local store."),
    Field::new("Created", "/createdTimestamp", FieldKind::ReadOnly),
    Field::new("Attributes", "/attributes", FieldKind::Attributes),
];

pub(crate) const USER_ACTIONS: &[Action] = &[
    Action::new("create", "Create user", Verb::Create, Danger::Confirm),
    Action::new(
        "assign-role",
        "Assign realm role",
        Verb::Special("assign-role"),
        Danger::Confirm,
    )
    .keywords(&["grant", "add role", "give permission", "rbac"]),
    Action::new(
        "unassign-role",
        "Remove realm role",
        Verb::Special("unassign-role"),
        Danger::Destructive,
    )
    .keywords(&["revoke", "take away role"]),
    Action::new(
        "join-group",
        "Add to group",
        Verb::Special("join-group"),
        Danger::Confirm,
    )
    .keywords(&["join", "membership", "put in group", "team"]),
    Action::new(
        "assign-client-role",
        "Assign client role",
        Verb::Special("pick-client-for-role"),
        Danger::Confirm,
    )
    .keywords(&["client role", "grant", "application role", "rbac"]),
    Action::new("edit", "Edit user", Verb::Edit, Danger::Confirm),
    Action::new("delete", "Delete user", Verb::Delete, Danger::Destructive),
    Action::new(
        "reset-password",
        "Set password",
        Verb::Special("reset-password"),
        Danger::Confirm,
    )
    .keywords(&["credential", "passwd", "temporary"]),
    Action::new(
        "logout",
        "Log out all sessions",
        Verb::Post("logout"),
        Danger::Confirm,
    )
    .keywords(&["revoke", "kill session", "sign out"]),
    Action::new(
        "impersonate",
        "Impersonate",
        Verb::Post("impersonation"),
        Danger::Destructive,
    )
    .needs(Requirement::Feature(features::IMPERSONATION)),
    Action::new(
        "send-actions-email",
        "Send required-actions e-mail",
        Verb::Special("execute-actions-email"),
        Danger::Confirm,
    )
    .keywords(&["verify email", "update password mail"]),
    Action::new(
        "unlock",
        "Clear brute-force lockout",
        Verb::Special("unlock-user"),
        Danger::Confirm,
    )
    .keywords(&["locked", "temporarily disabled", "attack detection"]),
];

pub(crate) const USERS: Resource = Resource {
    id: "users",
    singular: "User",
    plural: "Users",
    section: Section::Manage,
    collection: ep::USERS,
    item: Some(ep::USER),
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ID,
    new_object: Some(new_object::user),
    label: &["/username", "/email", "/id"],
    columns: USER_COLUMNS,
    search: Search::ServerWithAttributes,
    ops: Ops::CRUD,
    requirement: Requirement::Always,
    tabs: &[
        Tab::Form {
            title: "Details",
            fields: USER_FIELDS,
        },
        Tab::Nested {
            title: "Credentials",
            resource: "user-credentials",
            slot: "user",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Role mapping",
            resource: "role-mappings",
            slot: "owner",
            source: None,
            preset: &[("ownerKind", "users")],
        },
        Tab::Nested {
            title: "Client roles",
            resource: "client-role-mappings",
            slot: "owner",
            source: None,
            preset: &[("ownerKind", "users")],
        },
        Tab::Nested {
            title: "Groups",
            resource: "user-groups",
            slot: "user",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Sessions",
            resource: "user-sessions",
            slot: "user",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Identity links",
            resource: "user-identity-links",
            slot: "user",
            source: None,
            preset: &[],
        },
        Tab::Nested {
            title: "Consents",
            resource: "user-consents",
            slot: "user",
            source: None,
            preset: &[],
        },
        Tab::Raw,
    ],
    actions: USER_ACTIONS,
    keywords: &["account", "person", "member", "login", "identity"],
    summary: "Accounts in this realm, local or federated.",
};
