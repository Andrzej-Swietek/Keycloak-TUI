use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value;

use super::feature::{FeatureFlag, FeatureKind};

pub(super) type Catalog = BTreeMap<String, Arc<[String]>>;

pub(super) fn features(raw: &Value) -> BTreeMap<String, FeatureFlag> {
    raw.get("features")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let name = entry.get("name").and_then(Value::as_str)?.to_owned();
                    let flag = FeatureFlag {
                        label: entry
                            .get("label")
                            .and_then(Value::as_str)
                            .map(ToOwned::to_owned),
                        kind: entry
                            .get("type")
                            .and_then(Value::as_str)
                            .map_or(FeatureKind::Unknown(String::new()), FeatureKind::parse),
                        enabled: entry
                            .get("enabled")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                        name: name.clone(),
                    };
                    Some((name, flag))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `providers` is `{spi: {providers: {id: {...}}}}`.
pub(super) fn providers(raw: &Value) -> Catalog {
    raw.get("providers")
        .and_then(Value::as_object)
        .map(|spis| {
            spis.iter()
                .map(|(spi, detail)| {
                    let ids = detail
                        .get("providers")
                        .and_then(Value::as_object)
                        .map(|providers| providers.keys().cloned().collect::<Vec<_>>())
                        .unwrap_or_default();
                    (spi.clone(), Arc::from(ids))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `componentTypes` and `protocolMapperTypes` are both `{key: [{id, ...}]}`.
pub(super) fn id_lists(raw: &Value, field: &str) -> Catalog {
    raw.get(field)
        .and_then(Value::as_object)
        .map(|groups| {
            groups
                .iter()
                .map(|(key, entries)| {
                    let ids = entries
                        .as_array()
                        .map(|list| {
                            list.iter()
                                .filter_map(|entry| entry.get("id").and_then(Value::as_str))
                                .map(ToOwned::to_owned)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    (key.clone(), Arc::from(ids))
                })
                .collect()
        })
        .unwrap_or_default()
}
