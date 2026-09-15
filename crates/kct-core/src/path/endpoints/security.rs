use crate::path::Template;

pub const IDENTITY_PROVIDERS: Template =
    Template("/admin/realms/{realm}/identity-provider/instances");
pub const IDENTITY_PROVIDER: Template =
    Template("/admin/realms/{realm}/identity-provider/instances/{alias}");
pub const IDENTITY_PROVIDER_MAPPERS: Template =
    Template("/admin/realms/{realm}/identity-provider/instances/{alias}/mappers");
pub const IDENTITY_PROVIDER_MAPPER_TYPES: Template =
    Template("/admin/realms/{realm}/identity-provider/instances/{alias}/mapper-types");

pub const LDAP_TEST_CONNECTION: Template = Template("/admin/realms/{realm}/testLDAPConnection");
pub const LDAP_SYNC: Template = Template("/admin/realms/{realm}/user-storage/{component}/sync");
pub const LDAP_MAPPER_SYNC: Template =
    Template("/admin/realms/{realm}/user-storage/{component}/mappers/{mapper}/sync");
pub const LDAP_REMOVE_IMPORTED: Template =
    Template("/admin/realms/{realm}/user-storage/{component}/remove-imported-users");
pub const LDAP_UNLINK_USERS: Template =
    Template("/admin/realms/{realm}/user-storage/{component}/unlink-users");

pub const AUTH_FLOWS: Template = Template("/admin/realms/{realm}/authentication/flows");
pub const AUTH_FLOW: Template = Template("/admin/realms/{realm}/authentication/flows/{flow}");
pub const AUTH_FLOW_EXECUTIONS: Template =
    Template("/admin/realms/{realm}/authentication/flows/{flow}/executions");
pub const AUTH_FLOW_COPY: Template =
    Template("/admin/realms/{realm}/authentication/flows/{flow}/copy");
pub const AUTH_EXECUTION: Template =
    Template("/admin/realms/{realm}/authentication/executions/{execution}");
pub const AUTH_EXECUTION_CONFIG: Template =
    Template("/admin/realms/{realm}/authentication/executions/{execution}/config");
pub const AUTH_REQUIRED_ACTIONS: Template =
    Template("/admin/realms/{realm}/authentication/required-actions");
pub const AUTH_UNREGISTERED_REQUIRED_ACTIONS: Template =
    Template("/admin/realms/{realm}/authentication/unregistered-required-actions");
pub const AUTH_FORM_PROVIDERS: Template =
    Template("/admin/realms/{realm}/authentication/form-providers");

pub const AUTH_REQUIRED_ACTION: Template =
    Template("/admin/realms/{realm}/authentication/required-actions/{action}");
pub const IDENTITY_PROVIDER_MAPPER: Template =
    Template("/admin/realms/{realm}/identity-provider/instances/{alias}/mappers/{mapper}");

/// Adding to a flow: a step takes `{"provider": id}`, a sub-flow takes an alias and a type.
pub const AUTH_FLOW_ADD_EXECUTION: Template =
    Template("/admin/realms/{realm}/authentication/flows/{flow}/executions/execution");
pub const AUTH_FLOW_ADD_SUBFLOW: Template =
    Template("/admin/realms/{realm}/authentication/flows/{flow}/executions/flow");
pub const AUTH_AUTHENTICATOR_PROVIDERS: Template =
    Template("/admin/realms/{realm}/authentication/authenticator-providers");
