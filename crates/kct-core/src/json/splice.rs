use serde_json::Value;

use super::pointer;

/// `document` with the entry of `array` whose `key` equals `id` set to `element`.
///
/// `None` removes the entry; an `id` that is not present is appended. Everything else in
/// `document` is returned unchanged. A `document` that is not an object is replaced by one.
#[must_use]
pub fn splice(
    document: Value,
    array_at: &str,
    key: &str,
    id: &str,
    element: Option<Value>,
) -> Value {
    let mut document = match document {
        Value::Object(_) => document,
        _ => Value::Object(serde_json::Map::new()),
    };

    let current = pointer::at(&document, array_at)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let matches = |item: &Value| {
        pointer::at(item, key)
            .and_then(Value::as_str)
            .is_some_and(|found| found == id)
    };

    let mut replaced = false;
    let mut next: Vec<Value> = current
        .into_iter()
        .filter_map(|item| {
            if !matches(&item) {
                return Some(item);
            }
            replaced = true;
            element.clone()
        })
        .collect();

    if let (false, Some(added)) = (replaced, element) {
        next.push(added);
    }

    document = super::set_at(document, array_at, Value::Array(next));
    document
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn profile() -> Value {
        json!({
            "attributes": [
                {"name": "username", "displayName": "${username}"},
                {"name": "email", "displayName": "${email}"},
            ],
            "groups": [{"name": "user-metadata"}],
            "unmanagedAttributePolicy": "ENABLED",
        })
    }

    #[test]
    fn editing_one_entry_leaves_the_rest_of_the_document_alone() {
        let next = splice(
            profile(),
            "/attributes",
            "/name",
            "email",
            Some(json!({"name": "email", "displayName": "E-mail", "multivalued": false})),
        );

        assert_eq!(next["attributes"][1]["displayName"], json!("E-mail"));
        assert_eq!(next["attributes"][0]["name"], json!("username"));
        assert_eq!(next["groups"][0]["name"], json!("user-metadata"));
        assert_eq!(next["unmanagedAttributePolicy"], json!("ENABLED"));
    }

    #[test]
    fn an_unknown_id_is_appended_so_the_same_call_creates() {
        let next = splice(
            profile(),
            "/attributes",
            "/name",
            "department",
            Some(json!({"name": "department"})),
        );

        assert_eq!(next["attributes"].as_array().unwrap().len(), 3);
        assert_eq!(next["attributes"][2]["name"], json!("department"));
    }

    #[test]
    fn removing_an_entry_keeps_the_others_in_order() {
        let next = splice(profile(), "/attributes", "/name", "username", None);
        let names: Vec<&str> = next["attributes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|item| item["name"].as_str())
            .collect();

        assert_eq!(names, vec!["email"]);
    }

    #[test]
    fn a_document_without_the_array_yet_grows_one() {
        let next = splice(
            json!({}),
            "/attributes",
            "/name",
            "first",
            Some(json!({"name": "first"})),
        );
        assert_eq!(next["attributes"][0]["name"], json!("first"));
    }
}
