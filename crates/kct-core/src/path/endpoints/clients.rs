use crate::path::Template;

pub const CLIENTS: Template = Template("/admin/realms/{realm}/clients");
pub const CLIENT: Template = Template("/admin/realms/{realm}/clients/{client}");
pub const CLIENT_SECRET: Template =
    Template("/admin/realms/{realm}/clients/{client}/client-secret");
pub const CLIENT_ROLES: Template = Template("/admin/realms/{realm}/clients/{client}/roles");
pub const CLIENT_SERVICE_ACCOUNT: Template =
    Template("/admin/realms/{realm}/clients/{client}/service-account-user");
pub const CLIENT_SESSIONS: Template =
    Template("/admin/realms/{realm}/clients/{client}/user-sessions");
pub const CLIENT_SESSION_COUNT: Template =
    Template("/admin/realms/{realm}/clients/{client}/session-count");
pub const CLIENT_OFFLINE_SESSIONS: Template =
    Template("/admin/realms/{realm}/clients/{client}/offline-sessions");
pub const CLIENT_PROTOCOL_MAPPERS: Template =
    Template("/admin/realms/{realm}/clients/{client}/protocol-mappers/models");
pub const CLIENT_DEFAULT_SCOPES: Template =
    Template("/admin/realms/{realm}/clients/{client}/default-client-scopes");
pub const CLIENT_OPTIONAL_SCOPES: Template =
    Template("/admin/realms/{realm}/clients/{client}/optional-client-scopes");
pub const CLIENT_INSTALLATION: Template =
    Template("/admin/realms/{realm}/clients/{client}/installation/providers/{provider}");
pub const CLIENT_EVALUATE_SCOPES_MAPPERS: Template =
    Template("/admin/realms/{realm}/clients/{client}/evaluate-scopes/protocol-mappers");
pub const CLIENT_EVALUATE_TOKEN: Template = Template(
    "/admin/realms/{realm}/clients/{client}/evaluate-scopes/generate-example-access-token",
);

pub const CLIENT_AUTHZ_RESOURCES: Template =
    Template("/admin/realms/{realm}/clients/{client}/authz/resource-server/resource");
pub const CLIENT_AUTHZ_SCOPES: Template =
    Template("/admin/realms/{realm}/clients/{client}/authz/resource-server/scope");
pub const CLIENT_AUTHZ_POLICIES: Template =
    Template("/admin/realms/{realm}/clients/{client}/authz/resource-server/policy");
pub const CLIENT_AUTHZ_PERMISSIONS: Template =
    Template("/admin/realms/{realm}/clients/{client}/authz/resource-server/permission");
pub const CLIENT_AUTHZ_POLICY_PROVIDERS: Template =
    Template("/admin/realms/{realm}/clients/{client}/authz/resource-server/policy/providers");

pub const CLIENT_SCOPES: Template = Template("/admin/realms/{realm}/client-scopes");
pub const CLIENT_SCOPE: Template = Template("/admin/realms/{realm}/client-scopes/{scope}");
pub const CLIENT_SCOPE_PROTOCOL_MAPPERS: Template =
    Template("/admin/realms/{realm}/client-scopes/{scope}/protocol-mappers/models");
pub const REALM_DEFAULT_CLIENT_SCOPES: Template =
    Template("/admin/realms/{realm}/default-default-client-scopes");
pub const REALM_OPTIONAL_CLIENT_SCOPES: Template =
    Template("/admin/realms/{realm}/default-optional-client-scopes");

pub const ROLES: Template = Template("/admin/realms/{realm}/roles");
pub const ROLE_BY_NAME: Template = Template("/admin/realms/{realm}/roles/{role}");
pub const ROLE_BY_ID: Template = Template("/admin/realms/{realm}/roles-by-id/{role}");
pub const ROLE_COMPOSITES: Template =
    Template("/admin/realms/{realm}/roles-by-id/{role}/composites");
pub const ROLE_USERS: Template = Template("/admin/realms/{realm}/roles/{role}/users");
pub const ROLE_GROUPS: Template = Template("/admin/realms/{realm}/roles/{role}/groups");

/// Role mappings have the same shape for users and groups, so the owner kind is a slot.
pub const ROLE_MAPPINGS: Template =
    Template("/admin/realms/{realm}/{ownerKind}/{owner}/role-mappings");
pub const ROLE_MAPPINGS_REALM: Template =
    Template("/admin/realms/{realm}/{ownerKind}/{owner}/role-mappings/realm");
pub const ROLE_MAPPINGS_REALM_AVAILABLE: Template =
    Template("/admin/realms/{realm}/{ownerKind}/{owner}/role-mappings/realm/available");
pub const ROLE_MAPPINGS_CLIENT: Template =
    Template("/admin/realms/{realm}/{ownerKind}/{owner}/role-mappings/clients/{client}");
pub const ROLE_MAPPINGS_CLIENT_AVAILABLE: Template =
    Template("/admin/realms/{realm}/{ownerKind}/{owner}/role-mappings/clients/{client}/available");

/// Item endpoints for the sub-collections above. `roles-by-id` and `protocol-mappers/models`
/// do not follow the `collection/{id}` pattern.
pub const CLIENT_PROTOCOL_MAPPER: Template =
    Template("/admin/realms/{realm}/clients/{client}/protocol-mappers/models/{mapper}");
pub const CLIENT_AUTHZ_RESOURCE: Template =
    Template("/admin/realms/{realm}/clients/{client}/authz/resource-server/resource/{resource}");
pub const CLIENT_AUTHZ_SCOPE: Template =
    Template("/admin/realms/{realm}/clients/{client}/authz/resource-server/scope/{scope}");
pub const CLIENT_AUTHZ_POLICY: Template =
    Template("/admin/realms/{realm}/clients/{client}/authz/resource-server/policy/{policy}");
pub const CLIENT_AUTHZ_PERMISSION: Template = Template(
    "/admin/realms/{realm}/clients/{client}/authz/resource-server/permission/{permission}",
);
pub const CLIENT_INITIAL_ACCESS_ITEM: Template =
    Template("/admin/realms/{realm}/clients-initial-access/{token}");
pub const CLIENT_SCOPE_PROTOCOL_MAPPER: Template =
    Template("/admin/realms/{realm}/client-scopes/{scope}/protocol-mappers/models/{mapper}");
pub const CLIENT_AUTHZ_PERMISSION_PROVIDERS: Template =
    Template("/admin/realms/{realm}/clients/{client}/authz/resource-server/permission/providers");
pub const CLIENT_AUTHZ_RESOURCE_SERVER: Template =
    Template("/admin/realms/{realm}/clients/{client}/authz/resource-server");
