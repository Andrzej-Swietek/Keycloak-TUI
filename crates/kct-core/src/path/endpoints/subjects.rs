use crate::path::Template;

pub const USERS: Template = Template("/admin/realms/{realm}/users");
pub const USER: Template = Template("/admin/realms/{realm}/users/{user}");
pub const USERS_COUNT: Template = Template("/admin/realms/{realm}/users/count");
pub const USER_GROUPS: Template = Template("/admin/realms/{realm}/users/{user}/groups");
pub const USER_SESSIONS: Template = Template("/admin/realms/{realm}/users/{user}/sessions");
pub const USER_CREDENTIALS: Template = Template("/admin/realms/{realm}/users/{user}/credentials");
pub const USER_RESET_PASSWORD: Template =
    Template("/admin/realms/{realm}/users/{user}/reset-password");
pub const USER_LOGOUT: Template = Template("/admin/realms/{realm}/users/{user}/logout");
pub const USER_IMPERSONATION: Template =
    Template("/admin/realms/{realm}/users/{user}/impersonation");
pub const USER_FEDERATED_IDENTITY: Template =
    Template("/admin/realms/{realm}/users/{user}/federated-identity");
pub const USER_CONSENTS: Template = Template("/admin/realms/{realm}/users/{user}/consents");
pub const USER_EXECUTE_ACTIONS_EMAIL: Template =
    Template("/admin/realms/{realm}/users/{user}/execute-actions-email");
pub const USER_UNLOCK: Template =
    Template("/admin/realms/{realm}/attack-detection/brute-force/users/{user}");

pub const GROUPS: Template = Template("/admin/realms/{realm}/groups");
pub const GROUP: Template = Template("/admin/realms/{realm}/groups/{group}");
pub const GROUP_CHILDREN: Template = Template("/admin/realms/{realm}/groups/{group}/children");
pub const GROUP_MEMBERS: Template = Template("/admin/realms/{realm}/groups/{group}/members");
pub const GROUPS_COUNT: Template = Template("/admin/realms/{realm}/groups/count");

pub const ORGANIZATIONS: Template = Template("/admin/realms/{realm}/organizations");
pub const ORGANIZATION: Template = Template("/admin/realms/{realm}/organizations/{org}");
pub const ORGANIZATION_MEMBERS: Template =
    Template("/admin/realms/{realm}/organizations/{org}/members");
pub const ORGANIZATION_MEMBERS_COUNT: Template =
    Template("/admin/realms/{realm}/organizations/{org}/members/count");
pub const ORGANIZATION_IDPS: Template =
    Template("/admin/realms/{realm}/organizations/{org}/identity-providers");
