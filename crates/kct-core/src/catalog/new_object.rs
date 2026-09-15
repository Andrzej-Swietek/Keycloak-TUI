#![allow(clippy::must_use_candidate)]

use serde_json::{Value, json};

use crate::path::PathParams;

/// Builds the minimal body that creates one object of a resource.
///
/// A generic "name plus enabled" guess is wrong for most of Keycloak's representations —
/// `ComponentRepresentation` rejects `enabled` outright, a client scope needs a protocol, a
/// component needs its provider type and parent. Each resource therefore says what a valid
/// new object looks like, and gets the surrounding context in case it needs a parent id.
pub type NewObject = fn(&str, &PathParams) -> Value;

pub fn named(name: &str, _: &PathParams) -> Value {
    json!({ "name": name })
}

pub fn user(name: &str, _: &PathParams) -> Value {
    json!({ "username": name, "enabled": true })
}

pub fn group(name: &str, _: &PathParams) -> Value {
    json!({ "name": name })
}

pub fn organization(name: &str, _: &PathParams) -> Value {
    json!({ "name": name, "alias": name, "enabled": true })
}

pub fn client(name: &str, _: &PathParams) -> Value {
    json!({
        "clientId": name,
        "enabled": true,
        "protocol": "openid-connect",
        "publicClient": false,
        "standardFlowEnabled": true,
    })
}

pub fn client_scope(name: &str, _: &PathParams) -> Value {
    json!({
        "name": name,
        "protocol": "openid-connect",
        "attributes": {"include.in.token.scope": "true", "display.on.consent.screen": "true"},
    })
}

pub fn protocol_mapper(name: &str, _: &PathParams) -> Value {
    json!({
        "name": name,
        "protocol": "openid-connect",
        "protocolMapper": "oidc-usermodel-attribute-mapper",
        "config": {
            "user.attribute": "",
            "claim.name": name,
            "jsonType.label": "String",
            "access.token.claim": "true",
            "id.token.claim": "true",
        },
    })
}

pub fn identity_provider(name: &str, _: &PathParams) -> Value {
    json!({
        "alias": name,
        "providerId": "oidc",
        "enabled": true,
        // PKCE needs its method alongside the switch; "true" on its own is a 400.
        "config": {
            "clientAuthMethod": "client_secret_post",
            "pkceEnabled": "true",
            "pkceMethod": "S256",
        },
    })
}

pub fn realm(name: &str, _: &PathParams) -> Value {
    json!({ "realm": name, "enabled": true })
}

pub fn auth_flow(name: &str, _: &PathParams) -> Value {
    json!({
        "alias": name,
        "providerId": "basic-flow",
        "topLevel": true,
        "builtIn": false,
        "description": "",
    })
}

pub fn initial_access_token(_: &str, _: &PathParams) -> Value {
    json!({ "expiration": 0, "count": 1 })
}

/// An LDAP provider with the defaults Keycloak's own console starts from.
///
/// Two deliberate choices. `enabled` lives in `config`, not on the representation — putting
/// it on the representation is a 400. And it starts **disabled**: an enabled provider with a
/// blank connection URL is consulted by every user lookup in the realm and turns ordinary
/// operations into opaque 500s until someone notices.
pub fn ldap_provider(name: &str, params: &PathParams) -> Value {
    under_parent(
        json!({
        "name": name,
        "providerId": "ldap",
        "providerType": "org.keycloak.storage.UserStorageProvider",
        "config": {
            "enabled": ["false"],
            "vendor": ["other"],
            "connectionUrl": [""],
            "authType": ["simple"],
            "usersDn": [""],
            "usernameLDAPAttribute": ["uid"],
            "rdnLDAPAttribute": ["uid"],
            "uuidLDAPAttribute": ["entryUUID"],
            "userObjectClasses": ["inetOrgPerson, organizationalPerson"],
            "searchScope": ["2"],
            "editMode": ["READ_ONLY"],
            "importEnabled": ["true"],
            "cachePolicy": ["DEFAULT"],
        },
        }),
        params,
        "realmId",
    )
}

pub fn key_provider(name: &str, params: &PathParams) -> Value {
    under_parent(
        json!({
        "name": name,
        "providerId": "rsa-generated",
        "providerType": "org.keycloak.keys.KeyProvider",
        "config": {"priority": ["100"], "enabled": ["true"], "active": ["true"], "algorithm": ["RS256"]},
        }),
        params,
        "realmId",
    )
}

/// An LDAP mapper is a component whose parent is the provider it belongs to.
///
/// Both attribute names default to the mapper's own name: Keycloak rejects empty ones, and
/// mapping `department` to `department` is what an operator naming it that almost always
/// wants anyway.
pub fn ldap_mapper(name: &str, params: &PathParams) -> Value {
    under_parent(
        json!({
        "name": name,
        "providerId": "user-attribute-ldap-mapper",
        "providerType": "org.keycloak.storage.ldap.mappers.LDAPStorageMapper",
        "config": {
            "user.model.attribute": [name],
            "ldap.attribute": [name],
            "is.mandatory.in.ldap": ["false"],
            "read.only": ["true"],
            "always.read.value.from.ldap": ["false"],
        },
        }),
        params,
        "component",
    )
}

/// A broker mapper needs the provider it hangs off, which the parent screen supplies.
pub fn idp_mapper(name: &str, params: &PathParams) -> Value {
    json!({
        "name": name,
        "identityProviderAlias": params.get("alias").unwrap_or_default(),
        "identityProviderMapper": "oidc-user-attribute-idp-mapper",
        "config": {
            "syncMode": "INHERIT",
            "claim": name,
            "user.attribute": name,
        },
    })
}

/// `body` with `parentId` set from the `slot` path parameter, or unchanged when the parameter
/// is absent.
fn under_parent(body: Value, params: &PathParams, slot: &str) -> Value {
    match params.get(slot) {
        Some(parent) => crate::json::set_at(body, "/parentId", Value::String(parent.to_owned())),
        None => body,
    }
}
