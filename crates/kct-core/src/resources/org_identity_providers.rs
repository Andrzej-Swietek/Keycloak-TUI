use super::prelude::*;

/// This organization's link: 26.8 returns only the link to the organization asked about,
/// older releases a single domain in the provider's config.
pub(crate) fn organization_link(row: &Value) -> String {
    let Some(link) = row.pointer("/organizationLinks/0") else {
        return row
            .pointer("/config/kc.org.domain")
            .and_then(Value::as_str)
            .map(|domain| format!("domain {domain}"))
            .unwrap_or_default();
    };
    let kind = link
        .get("membershipType")
        .and_then(Value::as_str)
        .unwrap_or("UNMANAGED");
    match link.get("autoMembership").and_then(Value::as_bool) {
        Some(false) => format!("{} · no auto-join", kind.to_lowercase()),
        _ => kind.to_lowercase(),
    }
}

pub(crate) const ORG_IDENTITY_PROVIDERS: Resource = Resource {
    id: "org-identity-providers",
    singular: "Organization IdP",
    plural: "Organization identity providers",
    section: Section::Manage,
    collection: ep::ORGANIZATION_IDPS,
    item: None,
    shape: Shape::Array,
    fixed_params: &[],
    key: Key::ALIAS,
    new_object: None,
    label: &["/alias", "/displayName"],
    columns: &[
        Column::new("Alias", Projection::At("/alias"), Width::Flex(2)).toned(Tone::Primary),
        Column::new("Provider", Projection::At("/providerId"), Width::Fixed(14)),
        Column::new(
            "Enabled",
            Projection::Bool {
                pointer: "/enabled",
                yes: "yes",
                no: "DISABLED",
            },
            Width::Fixed(8),
        )
        .toned(Tone::Flag),
        Column::new(
            "Membership",
            Projection::Compute(organization_link),
            Width::Flex(3),
        ),
    ],
    search: Search::Client,
    ops: Ops::RO.with(Ops::CREATE).with(Ops::DELETE),
    requirement: KC_26,
    tabs: &[Tab::Raw],
    actions: &[
        Action::new(
            "membership",
            "Set membership type",
            Verb::Special("set-org-membership"),
            Danger::Confirm,
        )
        .needs(KC_26_8)
        .keywords(&["managed", "unmanaged", "auto membership"]),
        Action::new(
            "delete",
            "Unlink identity provider",
            Verb::Delete,
            Danger::Confirm,
        ),
    ],
    keywords: &["broker", "sso", "tenant idp"],
    summary: "Identity providers bound to this organization.",
};
