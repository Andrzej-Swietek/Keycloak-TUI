use kct_core::catalog::Verb;

use crate::model::{Model, Session};

/// Every action the catalog offers reaches a handler.
///
/// `Verb::Special` is dispatched by name, so a catalog entry and its handler can drift apart
/// without the compiler noticing: the action stays in the menu and answers "not wired up yet".
#[test]
fn every_special_action_is_dispatched() {
    let users = kct_core::resources::by_id("users").unwrap();
    let dead: Vec<String> = kct_core::resources::CATALOG
        .iter()
        .flat_map(|resource| {
            resource.actions.iter().filter_map(move |action| {
                let Verb::Special(name) = action.verb else {
                    return None;
                };
                let model = Model::new(Session::new("https://kc.test", "acme", "admin"), users);
                let said = super::specials::run(model, action)
                    .model
                    .toasts
                    .last()
                    .map(|toast| toast.notice.text.clone())
                    .unwrap_or_default();

                said.contains("not wired up")
                    .then(|| format!("{}/{name}", resource.id))
            })
        })
        .collect();

    assert!(
        dead.is_empty(),
        "these actions are offered and do nothing: {dead:#?}"
    );
}
