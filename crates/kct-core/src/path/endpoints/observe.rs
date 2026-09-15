use crate::path::Template;

pub const SESSIONS_BY_CLIENT: Template = Template("/admin/realms/{realm}/client-session-stats");
pub const SESSION_DELETE: Template = Template("/admin/realms/{realm}/sessions/{session}");
pub const LOGOUT_ALL: Template = Template("/admin/realms/{realm}/logout-all");
pub const PUSH_REVOCATION: Template = Template("/admin/realms/{realm}/push-revocation");

pub const EVENTS: Template = Template("/admin/realms/{realm}/events");
pub const ADMIN_EVENTS: Template = Template("/admin/realms/{realm}/admin-events");
pub const EVENTS_CONFIG: Template = Template("/admin/realms/{realm}/events/config");
pub const ATTACK_DETECTION: Template =
    Template("/admin/realms/{realm}/attack-detection/brute-force/users");

/// OIDC protocol endpoints; not part of the admin resource tree.
pub const OIDC_DISCOVERY: Template = Template("/realms/{realm}/.well-known/openid-configuration");
pub const OIDC_TOKEN: Template = Template("/realms/{realm}/protocol/openid-connect/token");
pub const OIDC_INTROSPECT: Template =
    Template("/realms/{realm}/protocol/openid-connect/token/introspect");
pub const OIDC_USERINFO: Template = Template("/realms/{realm}/protocol/openid-connect/userinfo");
pub const OIDC_LOGOUT: Template = Template("/realms/{realm}/protocol/openid-connect/logout");
pub const OIDC_CERTS: Template = Template("/realms/{realm}/protocol/openid-connect/certs");
pub const OIDC_DEVICE_AUTH: Template =
    Template("/realms/{realm}/protocol/openid-connect/auth/device");
