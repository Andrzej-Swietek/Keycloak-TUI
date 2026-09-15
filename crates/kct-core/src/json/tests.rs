use super::*;
use serde_json::json;

#[test]
fn merging_preserves_fields_this_build_has_never_heard_of() {
    let from_server = json!({
        "username": "alice",
        "enabled": true,
        "someFutureField": {"nested": [1, 2, 3]}
    });

    let merged = merge(from_server, json!({"enabled": false}));

    assert_eq!(merged["enabled"], json!(false));
    assert_eq!(merged["someFutureField"]["nested"], json!([1, 2, 3]));
}

#[test]
fn null_in_a_patch_deletes_and_arrays_replace_wholesale() {
    let merged = merge(
        json!({"a": 1, "b": 2, "uris": ["x", "y"]}),
        json!({"b": null, "uris": ["z"]}),
    );
    assert_eq!(merged, json!({"a": 1, "uris": ["z"]}));
}

#[test]
fn diff_then_merge_round_trips() {
    let before = json!({"keep": 1, "change": "old", "drop": true});
    let after = json!({"keep": 1, "change": "new", "added": 5});

    let patch = diff(&before, &after);
    assert_eq!(patch, json!({"change": "new", "added": 5, "drop": null}));
    assert_eq!(merge(before, patch), after);
}

#[test]
fn set_at_creates_missing_intermediate_objects() {
    let updated = set_at(json!({}), "/config/connectionUrl", json!("ldaps://dc1"));
    assert_eq!(updated["config"]["connectionUrl"], json!("ldaps://dc1"));
}

#[test]
fn attributes_unwrap_keycloaks_list_valued_shape() {
    let user = json!({"attributes": {"department": ["platform"], "plain": "v"}});
    assert_eq!(attribute(&user, "department").unwrap(), &json!("platform"));
    assert_eq!(attribute(&user, "plain").unwrap(), &json!("v"));
    assert!(attribute(&user, "missing").is_none());
}

#[test]
fn pointer_escaping_survives_keys_containing_slashes() {
    let value = json!({"a/b": {"c": 1}});
    let found = leaves(&value);
    assert_eq!(found[0].0, "/a~1b/c");
    assert_eq!(at(&value, &found[0].0).unwrap(), &json!(1));
}

#[test]
fn pruning_nulls_leaves_real_values_alone() {
    let pruned = prune_nulls(json!({"a": 1, "b": null, "c": {"d": null, "e": 2}}));
    assert_eq!(pruned, json!({"a": 1, "c": {"e": 2}}));
}
