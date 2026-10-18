use super::*;
use crate::catalog::{Ops, Tab};
use crate::path::PathParams;
use std::collections::BTreeSet;

#[test]
fn every_catalog_id_is_unique() {
    let ids: BTreeSet<_> = CATALOG.iter().map(|resource| resource.id).collect();
    assert_eq!(
        ids.len(),
        CATALOG.len(),
        "duplicate resource id in the catalog"
    );
}

#[test]
fn every_nested_tab_supplies_exactly_what_its_child_needs() {
    for resource in CATALOG {
        for tab in resource.tabs {
            let Tab::Nested {
                resource: child,
                slot,
                preset,
                ..
            } = tab
            else {
                continue;
            };
            let target = by_id(child)
                .unwrap_or_else(|| panic!("{} references unknown resource '{child}'", resource.id));

            let supplied: Vec<&str> = std::iter::once(*slot)
                .filter(|slot| !slot.is_empty())
                .chain(preset.iter().map(|(name, _)| *name))
                .collect();

            for needed in target.collection_slots() {
                assert!(
                    supplied.contains(&needed),
                    "{} opens '{child}' but never supplies slot '{needed}' (supplies {supplied:?})",
                    resource.id,
                );
            }

            // An empty slot says the child needs nothing from the parent — the two share a
            // document. It must not become a way to skip a slot the child really does need.
            if slot.is_empty() {
                assert_eq!(
                    target.collection_slots(),
                    resource.collection_slots(),
                    "{} opens '{child}' with no slot, but they do not share a path",
                    resource.id,
                );
                continue;
            }
            assert!(
                target.collection_slots().contains(slot),
                "{} passes slot '{slot}' to '{child}', which does not use it",
                resource.id,
            );
        }
    }
}

#[test]
fn every_nested_resource_is_reachable_from_some_parent() {
    let referenced: BTreeSet<&str> = CATALOG
        .iter()
        .flat_map(|resource| resource.tabs)
        .filter_map(|tab| match tab {
            Tab::Nested { resource, .. } => Some(*resource),
            Tab::Form { .. } | Tab::Raw => None,
        })
        .collect();

    for resource in CATALOG.iter().filter(|resource| resource.is_nested()) {
        assert!(
            referenced.contains(resource.id),
            "'{}' is nested but no tab opens it",
            resource.id
        );
    }
}

#[test]
fn every_listable_resource_can_render_a_row() {
    for resource in CATALOG {
        if resource.ops.has(Ops::LIST) && !resource.is_singleton() {
            assert!(
                !resource.columns.is_empty(),
                "{} declares no columns",
                resource.id
            );
            assert!(
                !resource.label.is_empty(),
                "{} declares no label",
                resource.id
            );
        }
    }
}

#[test]
fn every_resource_keeps_the_raw_json_escape_hatch() {
    for resource in CATALOG {
        assert!(
            resource.tabs.iter().any(|tab| matches!(tab, Tab::Raw)),
            "{} has no raw JSON tab",
            resource.id
        );
    }
}

#[test]
fn a_rows_own_id_and_its_parents_id_can_share_a_slot_name() {
    let children = by_id("group-children").unwrap();
    assert_eq!(children.collection_slots(), vec!["group"]);
    assert_eq!(children.item_slot(), Some("group"));

    let mappers = by_id("federation-mappers").unwrap();
    assert_eq!(mappers.collection_slots(), vec!["component"]);
    assert_eq!(mappers.item_slot(), Some("component"));

    assert!(by_id("users").unwrap().collection_slots().is_empty());
    assert_eq!(by_id("users").unwrap().item_slot(), Some("user"));
}

#[test]
fn nested_component_collections_resolve_their_parent_filter() {
    let mappers = by_id("federation-mappers").unwrap();
    let params = PathParams::realm("acme").with("component", "ldap-1");
    let resolved = mappers.resolved_params(&params);

    assert!(resolved.contains(&("parent".to_owned(), "ldap-1".to_owned())));
    assert!(
        resolved
            .iter()
            .any(|(k, v)| k == "type" && v.contains("LDAPStorageMapper"))
    );
}

#[test]
fn top_level_excludes_things_that_need_a_parent() {
    let top: Vec<_> = top_level().map(|resource| resource.id).collect();

    assert!(top.contains(&"users"));
    assert!(top.contains(&"user-federation"));
    assert!(!top.contains(&"client-roles"), "client roles need a client");
    assert!(!top.contains(&"federation-mappers"));
}

#[test]
fn organizations_are_gated_on_version_and_feature() {
    let organizations = by_id("organizations").unwrap();
    let old = crate::capability::ServerProfile::from_discovery(crate::Version::new(25, 0, 6));

    assert!(!old.evaluate(&organizations.requirement).is_permitted());
}

#[test]
fn client_type_is_derived_the_way_keycloak_means_it() {
    use super::client_fields::client_kind;

    assert_eq!(
        client_kind(&serde_json::json!({"publicClient": true})),
        "public"
    );
    assert_eq!(
        client_kind(&serde_json::json!({"publicClient": false, "serviceAccountsEnabled": true})),
        "confidential +svc"
    );
    assert_eq!(
        client_kind(&serde_json::json!({"bearerOnly": true, "publicClient": true})),
        "bearer-only"
    );
}

#[test]
fn mapper_surfaces_explain_a_missing_claim_at_a_glance() {
    use super::shared::mapper_surfaces;

    let mapper = serde_json::json!({
        "config": {
            "access.token.claim": "true",
            "id.token.claim": "false",
            "userinfo.token.claim": "true"
        }
    });
    assert_eq!(mapper_surfaces(&mapper), "access,userinfo");
}
