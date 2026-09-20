mod vocabulary;

use kct_core::catalog::{Field, FieldKind, Resource};
use vocabulary as vocab;

use crate::model::Session;

/// Values a field is known to accept.
///
/// Three sources, in order of confidence: the catalog's own enumeration, the live server
/// (installed themes, registered providers, this realm's flows), and the short vocabularies
/// Keycloak documents but does not expose. Anything unknown gets no suggestions — an empty
/// list is honest, a wrong one wastes an operator's afternoon.
#[must_use]
pub fn for_field(field: &Field, resource: &'static Resource, session: &Session) -> Vec<String> {
    if let FieldKind::Choice(options) = field.kind {
        return options.iter().map(|option| (*option).to_owned()).collect();
    }
    if vocab::FLOW_BINDINGS.contains(&field.pointer) {
        return session.auth_flows.to_vec();
    }

    match field.pointer {
        "/loginTheme" => session.profile.themes("login"),
        "/accountTheme" => session.profile.themes("account"),
        "/adminTheme" => session.profile.themes("admin"),
        "/emailTheme" => session.profile.themes("email"),
        "/protocolMapper" => session.profile.mapper_types("openid-connect"),
        "/identityProviderMapper" => session.profile.provider_ids("identity-provider-mapper"),
        "/providerId" => provider_for(resource, session),
        "/clientAuthenticatorType" => {
            or_else(session.profile.provider_ids("client-authenticator"), &[])
        }
        "/defaultSignatureAlgorithm"
        | "/algorithm"
        | "/attributes/access.token.signed.response.alg"
        | "/attributes/id.token.signed.response.alg" => or_else(
            session.profile.provider_ids("signature"),
            vocab::SIGNATURE_ALGORITHMS,
        ),
        "/requiredActions" | "/providerId/required-action" | "/defaultAction" => or_else(
            session.profile.provider_ids("required-action"),
            vocab::REQUIRED_ACTIONS,
        ),
        "/passwordPolicy" => session.profile.password_policies(),
        "/eventsListeners" => session.profile.provider_ids("eventsListener"),
        "/enabledEventTypes" => session.profile.event_types(),
        "/protocol" => listed(vocab::PROTOCOLS),
        "/otpPolicyType" => listed(vocab::OTP_TYPES),
        "/otpPolicyAlgorithm" => listed(vocab::OTP_ALGORITHMS),
        "/otpPolicyDigits" => listed(vocab::OTP_DIGITS),
        "/sslRequired" => listed(vocab::SSL_REQUIRED),
        "/publicClient" | "/bearerOnly" => listed(vocab::ACCESS_TYPES),
        "/supportedLocales" | "/defaultLocale" => listed(vocab::LOCALES),
        _ => Vec::new(),
    }
}

/// `providerId` means a different SPI on every screen that has one.
fn provider_for(resource: &'static Resource, session: &Session) -> Vec<String> {
    let spi = match resource.id {
        "identity-providers" | "idp-mappers" => "identity-provider",
        "user-federation" => "org.keycloak.storage.UserStorageProvider",
        "key-providers" => "org.keycloak.keys.KeyProvider",
        "required-actions" => "required-action",
        "client-policies" | "client-profiles" => "client-policy-executor",
        _ => return Vec::new(),
    };
    session.profile.provider_ids(spi)
}

fn listed(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

/// Prefer what the server reports; fall back to the documented list only when it says nothing.
fn or_else(live: Vec<String>, fallback: &[&str]) -> Vec<String> {
    if live.is_empty() {
        listed(fallback)
    } else {
        live
    }
}

/// Narrow suggestions by what has been typed so far, keeping the declared order.
#[must_use]
pub fn matching(all: &[String], typed: &str) -> Vec<usize> {
    let needle = typed.trim().to_lowercase();
    all.iter()
        .enumerate()
        .filter(|(_, value)| needle.is_empty() || value.to_lowercase().contains(&needle))
        .map(|(index, _)| index)
        .collect()
}

#[cfg(test)]
mod tests;
