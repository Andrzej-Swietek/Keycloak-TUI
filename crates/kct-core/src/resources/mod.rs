//! The resource catalog: every Keycloak collection this tool can drive.
//!
//! Read these modules as documentation of the admin API surface. Each entry is pure data, so
//! it can be checked against Keycloak's REST reference without reading any UI code, and an
//! upgrade that moves an endpoint is a diff confined to one file.

mod authentication;
mod authz;
mod client_fields;
mod client_policies;
mod client_policies_global;
mod client_registration;
mod client_scope_fields;
mod client_scopes_sub;
mod client_subs;
mod clients;
mod federation;
mod flow_executions;
mod groups;
mod identity_providers;
mod ldap_fields;
mod members;
mod observe;
mod organizations;
mod policies;
mod prelude;
mod realm;
mod realm_config;
mod realm_fields;
mod realm_flow_fields;
mod realm_runtime_fields;
mod realm_webauthn_fields;
mod registration;
mod role_mappings;
mod roles;
mod scopes;
mod server;
mod shared;
mod user_links;
mod user_profile;
mod user_subs;
mod users;

use crate::catalog::{Resource, Section};

/// Every resource, in navigation order. Nested entries follow the top-level ones.
pub static CATALOG: &[Resource] = &[
    users::USERS,
    groups::GROUPS,
    organizations::ORGANIZATIONS,
    clients::CLIENTS,
    scopes::CLIENT_SCOPES,
    roles::REALM_ROLES,
    identity_providers::IDENTITY_PROVIDERS,
    federation::USER_FEDERATION,
    authentication::AUTH_FLOWS,
    registration::REQUIRED_ACTIONS,
    registration::CLIENT_INITIAL_ACCESS,
    client_registration::CLIENT_REGISTRATION_POLICIES,
    realm::REALM_SETTINGS,
    realm_config::REALM_KEYS,
    realm_config::KEY_PROVIDERS,
    realm_config::USER_PROFILE,
    client_policies::CLIENT_POLICY_ITEMS,
    client_policies::CLIENT_PROFILE_ITEMS,
    client_policies_global::CLIENT_POLICIES_GLOBAL,
    client_policies_global::CLIENT_PROFILES_GLOBAL,
    policies::EVENT_CONFIG,
    observe::CLIENT_SESSION_STATS,
    observe::LOGIN_EVENTS,
    observe::ADMIN_EVENTS,
    realm::REALMS,
    server::SERVER_FEATURES,
    server::SERVER_PROVIDERS,
    server::SERVER_INFO,
    client_subs::CLIENT_ROLES,
    client_subs::CLIENT_MAPPERS,
    client_subs::CLIENT_SESSIONS,
    client_scopes_sub::CLIENT_DEFAULT_SCOPES,
    client_scopes_sub::CLIENT_OPTIONAL_SCOPES,
    client_scopes_sub::CLIENT_EFFECTIVE_MAPPERS,
    authz::CLIENT_AUTHZ_RESOURCES,
    authz::CLIENT_AUTHZ_SCOPES,
    authz::CLIENT_AUTHZ_POLICIES,
    authz::CLIENT_AUTHZ_PERMISSIONS,
    scopes::SCOPE_MAPPERS,
    identity_providers::IDP_MAPPERS,
    federation::FEDERATION_MAPPERS,
    flow_executions::FLOW_EXECUTIONS,
    groups::GROUP_CHILDREN,
    members::GROUP_MEMBERS,
    members::ORG_MEMBERS,
    organizations::ORG_IDENTITY_PROVIDERS,
    roles::ROLE_USERS,
    roles::ROLE_GROUPS,
    role_mappings::ROLE_MAPPINGS,
    role_mappings::CLIENT_ROLE_MAPPINGS,
    user_subs::USER_CREDENTIALS,
    user_subs::USER_GROUPS,
    user_subs::USER_SESSIONS,
    user_links::USER_CONSENTS,
    user_links::USER_IDENTITY_LINKS,
    user_profile::USER_PROFILE_ATTRIBUTES,
    user_profile::USER_PROFILE_GROUPS,
];

#[must_use]
pub fn by_id(id: &str) -> Option<&'static Resource> {
    CATALOG.iter().find(|resource| resource.id == id)
}

/// Resources belonging to a section, in catalog order.
pub fn in_section(section: Section) -> impl Iterator<Item = &'static Resource> {
    CATALOG
        .iter()
        .filter(move |resource| resource.section == section)
}

/// Resources reachable directly from the navigation tree.
pub fn top_level() -> impl Iterator<Item = &'static Resource> {
    CATALOG.iter().filter(|resource| !resource.is_nested())
}

#[cfg(test)]
mod tests;
