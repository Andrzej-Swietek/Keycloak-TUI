use crate::effect::Effect;
use crate::model::Model;

/// The result of one `update` step: the next state, and the I/O it asks for.
#[derive(Debug)]
pub struct Transition {
    pub model: Model,
    pub effects: Vec<Effect>,
}

impl Transition {
    #[must_use]
    pub fn pure(model: Model) -> Self {
        Self {
            model,
            effects: Vec::new(),
        }
    }

    #[must_use]
    pub fn with(model: Model, effect: Effect) -> Self {
        Self {
            model,
            effects: vec![effect],
        }
    }

    #[must_use]
    pub fn many(model: Model, effects: Vec<Effect>) -> Self {
        Self { model, effects }
    }

    /// Append one effect, keeping the model.
    #[must_use]
    pub fn and(self, effect: Effect) -> Self {
        let mut effects = self.effects;
        effects.push(effect);
        Self { effects, ..self }
    }

    /// Run `step` on the resulting model, keeping the effects of both in order.
    #[must_use]
    pub fn and_then(self, step: impl FnOnce(Model) -> Self) -> Self {
        let next = step(self.model);
        Self {
            model: next.model,
            effects: self.effects.into_iter().chain(next.effects).collect(),
        }
    }

    /// Replace the model, keeping the effects.
    #[must_use]
    pub fn map(self, change: impl FnOnce(Model) -> Model) -> Self {
        Self {
            model: change(self.model),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Model, Session};

    fn model() -> Model {
        Model::new(
            Session::new("https://kc.test", "master", "admin"),
            kct_core::resources::by_id("users").unwrap(),
        )
    }

    #[test]
    fn chaining_keeps_both_steps_effects_in_order() {
        let chained = Transition::with(model(), Effect::ListRealms)
            .and_then(|model| Transition::with(model, Effect::Quit));

        assert!(matches!(
            chained.effects.as_slice(),
            [Effect::ListRealms, Effect::Quit]
        ));
    }

    #[test]
    fn mapping_leaves_the_effects_alone() {
        let mapped = Transition::with(model(), Effect::Quit).map(|model| Model {
            viewport_rows: 42,
            ..model
        });

        assert_eq!(mapped.model.viewport_rows, 42);
        assert_eq!(mapped.effects.len(), 1);
    }
}
