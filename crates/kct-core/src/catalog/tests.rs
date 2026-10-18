use super::*;
use serde_json::json;

#[test]
fn ops_compose_as_a_bitset() {
    assert!(Ops::CRUD.has(Ops::DELETE));
    assert!(!Ops::RO.has(Ops::DELETE));
    assert!(Ops::RO.with(Ops::CREATE).has(Ops::CREATE));
    assert!(!Ops::CRUD.without(Ops::DELETE).has(Ops::DELETE));
}

#[test]
fn projections_never_fail_on_missing_data() {
    let row = json!({"enabled": true, "members": [1, 2, 3], "attributes": {"tier": ["gold"]}});

    assert_eq!(
        Projection::Bool {
            pointer: "/enabled",
            yes: "on",
            no: "off"
        }
        .render(&row),
        "on"
    );
    assert_eq!(Projection::Count("/members").render(&row), "3");
    assert_eq!(Projection::Attribute("tier").render(&row), "gold");
    assert_eq!(Projection::At("/nope").render(&row), "");
    assert_eq!(Projection::Count("/nope").render(&row), "");
}

#[test]
fn at_any_survives_a_renamed_field() {
    let projection = Projection::AtAny(&["/name", "/legacyName"]);
    assert_eq!(projection.render(&json!({"legacyName": "x"})), "x");
    assert_eq!(projection.render(&json!({"name": "x"})), "x");
}

#[test]
fn the_three_keycloak_encodings_round_trip() {
    let native = Field::new("Enabled", "/enabled", FieldKind::Bool);
    let object = native.write(json!({}), json!(true));
    assert_eq!(object, json!({"enabled": true}));
    assert_eq!(native.read(&object), Some(json!(true)));

    let stringly = Field::stringly(
        "Lightweight",
        "/attributes/client.use.lightweight.access.token.enabled",
        FieldKind::Bool,
    );
    let object = stringly.write(json!({}), json!(true));
    assert_eq!(
        object["attributes"]["client.use.lightweight.access.token.enabled"],
        json!("true"),
        "client attributes must be strings or Keycloak silently ignores them"
    );
    assert_eq!(stringly.read(&object), Some(json!(true)));

    let config = Field::config("Connection URL", "/config/connectionUrl", FieldKind::Text);
    let object = config.write(json!({}), json!("ldaps://dc1:636"));
    assert_eq!(
        object["config"]["connectionUrl"],
        json!(["ldaps://dc1:636"])
    );
    assert_eq!(config.read(&object), Some(json!("ldaps://dc1:636")));
}

#[test]
fn secrets_are_never_rendered_in_clear_text() {
    let secret = Field::config(
        "Bind credential",
        "/config/bindCredential",
        FieldKind::Secret,
    );
    let shown = secret.display(&json!({"config": {"bindCredential": ["hunter2"]}}));

    assert!(!shown.contains("hunter2"));
    assert_eq!(shown, "•••••••");
    assert_eq!(
        secret.display(&json!({"config": {"bindCredential": [""]}})),
        "",
        "an unset secret reads as unset, not as a mask over nothing"
    );
}

#[test]
fn durations_render_as_something_a_human_can_check() {
    let field = Field::new("Lifespan", "/accessTokenLifespan", FieldKind::Seconds);
    assert_eq!(field.display(&json!({"accessTokenLifespan": 300})), "5m");
    assert_eq!(field.display(&json!({"accessTokenLifespan": 86_400})), "1d");
    assert_eq!(
        field.display(&json!({"accessTokenLifespan": 5_400})),
        "1h 30m"
    );
}

#[test]
fn shapes_normalise_every_response_layout_keycloak_uses() {
    assert_eq!(Shape::Array.rows(json!([{"a": 1}])).len(), 1);
    assert_eq!(
        Shape::ArrayAt("/keys")
            .rows(json!({"keys": [{"kid": "k1"}, {"kid": "k2"}]}))
            .len(),
        2
    );

    let entries =
        Shape::EntriesAt("/providers").rows(json!({"providers": {"policy": {"internal": false}}}));
    assert_eq!(entries[0]["name"], json!("policy"));
    assert_eq!(entries[0]["internal"], json!(false));

    assert_eq!(
        Shape::ArrayAt("/keys").rows(json!({"unexpected": 1})),
        Vec::<serde_json::Value>::new()
    );
}
