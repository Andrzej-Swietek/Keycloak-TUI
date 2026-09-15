use crate::path::Template;

pub const SERVER_INFO: Template = Template("/admin/serverinfo");
pub const REALMS: Template = Template("/admin/realms");
pub const REALM: Template = Template("/admin/realms/{realm}");

pub const KEYS: Template = Template("/admin/realms/{realm}/keys");
pub const COMPONENTS: Template = Template("/admin/realms/{realm}/components");
pub const COMPONENT: Template = Template("/admin/realms/{realm}/components/{component}");
pub const COMPONENT_SUBTYPES: Template =
    Template("/admin/realms/{realm}/components/{component}/sub-component-types");

pub const PARTIAL_EXPORT: Template = Template("/admin/realms/{realm}/partial-export");
pub const PARTIAL_IMPORT: Template = Template("/admin/realms/{realm}/partialImport");
pub const USER_PROFILE: Template = Template("/admin/realms/{realm}/users/profile");

pub const CLIENT_POLICIES_POLICIES: Template =
    Template("/admin/realms/{realm}/client-policies/policies");
pub const CLIENT_POLICIES_PROFILES: Template =
    Template("/admin/realms/{realm}/client-policies/profiles");
pub const CLIENT_REGISTRATION_POLICIES: Template =
    Template("/admin/realms/{realm}/client-registration-policy/providers");
pub const CLIENT_INITIAL_ACCESS: Template =
    Template("/admin/realms/{realm}/clients-initial-access");
