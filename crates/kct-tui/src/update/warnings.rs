use kct_core::json;
use serde_json::Value;

use crate::model::{Model, UnmanagedAttributes};

/// Things worth telling an operator *before* a write, not after.
///
/// The motivating case is the declarative user profile: since Keycloak 24, an attribute the
/// profile does not declare is discarded on write and the request still returns success. The
/// attribute simply is not there afterwards, with nothing in the response to say so.
#[must_use]
pub fn before_save(model: &Model, patch: &Value) -> Vec<String> {
    let mut warnings = Vec::new();

    if model.resource().id == "users" {
        warnings.extend(undeclared_attributes(model, patch));
    }
    if model.session.dry_run {
        warnings.push("dry-run is on: this will not be sent".to_owned());
    }

    warnings
}

fn undeclared_attributes(model: &Model, patch: &Value) -> Option<String> {
    if model.session.unmanaged_attributes != UnmanagedAttributes::Discarded {
        return None;
    }

    let touched: Vec<&String> = json::at(patch, "/attributes")
        .and_then(Value::as_object)
        .map(|attributes| {
            attributes
                .keys()
                .filter(|name| !model.session.declared_attributes.contains(name))
                .collect()
        })
        .unwrap_or_default();

    (!touched.is_empty()).then(|| {
        format!(
            "This realm's user profile declares no policy for unmanaged attributes, so \
             Keycloak will silently discard {}. Set an unmanaged-attribute policy under \
             Realm ▸ User profile, or declare the attribute there first.",
            touched
                .iter()
                .map(|name| format!("'{name}'"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Session;
    use serde_json::json;
    use std::sync::Arc;

    fn model(policy: UnmanagedAttributes, declared: &[&str]) -> Model {
        let users = kct_core::resources::by_id("users").unwrap();
        Model {
            session: Session {
                unmanaged_attributes: policy,
                declared_attributes: Arc::from(
                    declared
                        .iter()
                        .map(|name| (*name).to_owned())
                        .collect::<Vec<_>>(),
                ),
                ..Session::new("https://kc", "master", "admin")
            },
            ..Model::new(Session::new("https://kc", "master", "admin"), users)
        }
    }

    #[test]
    fn an_attribute_the_profile_would_discard_is_flagged_by_name() {
        let model = model(UnmanagedAttributes::Discarded, &["email"]);
        let warnings = before_save(&model, &json!({"attributes": {"department": ["platform"]}}));

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("'department'"));
        assert!(warnings[0].contains("User profile"));
    }

    #[test]
    fn a_declared_attribute_is_not_flagged() {
        let model = model(UnmanagedAttributes::Discarded, &["department"]);
        assert!(before_save(&model, &json!({"attributes": {"department": ["x"]}})).is_empty());
    }

    #[test]
    fn a_realm_that_allows_unmanaged_attributes_needs_no_warning() {
        let model = model(UnmanagedAttributes::Allowed, &[]);
        assert!(before_save(&model, &json!({"attributes": {"anything": ["x"]}})).is_empty());
    }

    #[test]
    fn edits_that_do_not_touch_attributes_are_silent() {
        let model = model(UnmanagedAttributes::Discarded, &[]);
        assert!(before_save(&model, &json!({"enabled": false})).is_empty());
    }
}
