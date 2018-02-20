use state;
use state::State;
use world::World;
use doormax::effect::{Attribute, Effect};

#[derive(Debug, Clone, Default)]
pub struct EffectLearner {
    effects: Vec<Effect>,
}

impl EffectLearner {
    pub fn new() -> Self {
        EffectLearner {
            effects: Vec::new(),
        }
    }

    pub fn predict(
        &self,
        attribute: Attribute,
        world: &World,
        state: &State,
    ) -> Result<Option<State>, state::Error> {
        // Notice that this returns None (aka bottom)
        // if effects is empty.
        let mut predicted_state = None;

        for e in &self.effects {
            let modified_state = e.apply(attribute, world, state)?;
            match predicted_state {
                None => predicted_state = Some(modified_state),
                Some(previous_prediction) => if previous_prediction != modified_state {
                    return Ok(None);
                },
            }
        }

        Ok(predicted_state)
    }

    pub fn apply_experience(&mut self, effects: &[Effect]) {
        if self.effects.is_empty() {
            self.effects = effects.to_vec();
        } else {
            self.effects.retain(|e| effects.contains(e));
        }
    }
}

#[cfg(test)]
mod effect_learner_test {
    use super::*;

    #[test]
    fn learns_taxi_x_east() {
        let source_world = "\
                            ┌───┬─────┐\n\
                            │R .│. . .│\n\
                            │   │     │\n\
                            │. .│G . .│\n\
                            │         │\n\
                            │. . . . .│\n\
                            │         │\n\
                            │.│Y .│B .│\n\
                            │ │   │   │\n\
                            │.│. .│. .│\n\
                            └─┴───┴───┘\n\
                            ";

        let w = World::build_from_str(source_world).unwrap();

        let old_state = State::build(&w, (1, 3), Some('R'), 'B').unwrap();
        let new_state = State::build(&w, (2, 3), Some('R'), 'B').unwrap();

        let mut learner = EffectLearner::new();
        let test_state = State::build(&w, (3, 3), Some('R'), 'B').unwrap();

        let predicted_0 = learner.predict(Attribute::TaxiX, &w, &test_state).unwrap();
        assert_eq!(predicted_0, None);

        learner.apply_experience(&Effect::generate_effects(
            Attribute::TaxiX,
            &w,
            &old_state,
            &new_state,
        ));

        let expected_1 = Some(State::build(&w, (4, 3), Some('R'), 'B').unwrap());
        let predicted_1 = learner.predict(Attribute::TaxiX, &w, &test_state).unwrap();
        assert_eq!(predicted_1, expected_1);

        let test_failure = State::build(&w, (4, 3), Some('R'), 'B').unwrap();

        let predicted_failure = learner.predict(Attribute::TaxiX, &w, &test_failure);
        assert!(predicted_failure.is_err());
    }

    #[test]
    fn learns_taxi_x_west() {
        let source_world = "\
                            ┌───┬─────┐\n\
                            │R .│. . .│\n\
                            │   │     │\n\
                            │. .│G . .│\n\
                            │         │\n\
                            │. . . . .│\n\
                            │         │\n\
                            │.│Y .│B .│\n\
                            │ │   │   │\n\
                            │.│. .│. .│\n\
                            └─┴───┴───┘\n\
                            ";

        let w = World::build_from_str(source_world).unwrap();

        let old_state = State::build(&w, (2, 3), Some('R'), 'B').unwrap();
        let new_state = State::build(&w, (1, 3), Some('R'), 'B').unwrap();

        let mut learner = EffectLearner::new();
        let test_state = State::build(&w, (3, 3), Some('R'), 'B').unwrap();

        let predicted_0 = learner.predict(Attribute::TaxiX, &w, &test_state).unwrap();
        assert_eq!(predicted_0, None);

        learner.apply_experience(&Effect::generate_effects(
            Attribute::TaxiX,
            &w,
            &old_state,
            &new_state,
        ));

        let expected_1 = Some(State::build(&w, (2, 3), Some('R'), 'B').unwrap());
        let predicted_1 = learner.predict(Attribute::TaxiX, &w, &test_state).unwrap();
        assert_eq!(predicted_1, expected_1);
    }

    #[test]
    fn unlearns_taxi_x() {
        let source_world = "\
                            ┌───┬─────┐\n\
                            │R .│. . .│\n\
                            │   │     │\n\
                            │. .│G . .│\n\
                            │         │\n\
                            │. . . . .│\n\
                            │         │\n\
                            │.│Y .│B .│\n\
                            │ │   │   │\n\
                            │.│. .│. .│\n\
                            └─┴───┴───┘\n\
                            ";

        let w = World::build_from_str(source_world).unwrap();

        let old_state = State::build(&w, (1, 3), Some('R'), 'B').unwrap();
        let new_state = State::build(&w, (2, 3), Some('R'), 'B').unwrap();

        let mut learner = EffectLearner::new();

        learner.apply_experience(&Effect::generate_effects(
            Attribute::TaxiX,
            &w,
            &old_state,
            &new_state,
        ));

        let test_state = State::build(&w, (3, 3), Some('R'), 'B').unwrap();

        let expected_0 = Some(State::build(&w, (4, 3), Some('R'), 'B').unwrap());
        let predicted_0 = learner.predict(Attribute::TaxiX, &w, &test_state).unwrap();
        assert_eq!(predicted_0, expected_0);

        let conflicting_old = State::build(&w, (1, 2), Some('R'), 'B').unwrap();
        let conflicting_new = State::build(&w, (0, 2), Some('R'), 'B').unwrap();

        learner.apply_experience(&Effect::generate_effects(
            Attribute::TaxiX,
            &w,
            &conflicting_old,
            &conflicting_new,
        ));

        let predicted_1 = learner.predict(Attribute::TaxiX, &w, &test_state).unwrap();
        assert_eq!(predicted_1, None);
    }

    #[test]
    fn learns_taxi_x_nothing() {
        let source_world = "\
                            ┌───┬─────┐\n\
                            │R .│. . .│\n\
                            │   │     │\n\
                            │. .│G . .│\n\
                            │         │\n\
                            │. . . . .│\n\
                            │         │\n\
                            │.│Y .│B .│\n\
                            │ │   │   │\n\
                            │.│. .│. .│\n\
                            └─┴───┴───┘\n\
                            ";

        let w = World::build_from_str(source_world).unwrap();

        let old_state = State::build(&w, (1, 3), Some('R'), 'B').unwrap();
        let new_state = State::build(&w, (1, 4), None, 'B').unwrap();

        let mut learner = EffectLearner::new();
        let test_state = State::build(&w, (3, 3), Some('R'), 'B').unwrap();

        let predicted_0 = learner.predict(Attribute::TaxiX, &w, &test_state).unwrap();
        assert_eq!(predicted_0, None);

        learner.apply_experience(&Effect::generate_effects(
            Attribute::TaxiX,
            &w,
            &old_state,
            &new_state,
        ));

        let predicted_1 = learner.predict(Attribute::TaxiX, &w, &test_state).unwrap();
        assert_eq!(predicted_1, None);
    }

    #[test]
    fn learns_taxi_y() {
        let source_world = "\
                            ┌───┬─────┐\n\
                            │R .│. . .│\n\
                            │   │     │\n\
                            │. .│G . .│\n\
                            │         │\n\
                            │. . . . .│\n\
                            │         │\n\
                            │.│Y .│B .│\n\
                            │ │   │   │\n\
                            │.│. .│. .│\n\
                            └─┴───┴───┘\n\
                            ";

        let w = World::build_from_str(source_world).unwrap();

        let old_state = State::build(&w, (2, 3), Some('R'), 'B').unwrap();
        let new_state = State::build(&w, (2, 2), Some('R'), 'B').unwrap();

        let mut learner = EffectLearner::new();
        let test_state = State::build(&w, (3, 3), Some('R'), 'B').unwrap();

        let predicted_0 = learner.predict(Attribute::TaxiY, &w, &test_state).unwrap();
        assert_eq!(predicted_0, None);

        learner.apply_experience(&Effect::generate_effects(
            Attribute::TaxiY,
            &w,
            &old_state,
            &new_state,
        ));

        let expected_1 = Some(State::build(&w, (3, 2), Some('R'), 'B').unwrap());
        let predicted_1 = learner.predict(Attribute::TaxiY, &w, &test_state).unwrap();
        assert_eq!(predicted_1, expected_1);
    }

    #[test]
    fn learns_passenger_pickup() {
        let source_world = "\
                            ┌───┬─────┐\n\
                            │R .│. . .│\n\
                            │   │     │\n\
                            │. .│G . .│\n\
                            │         │\n\
                            │. . . . .│\n\
                            │         │\n\
                            │.│Y .│B .│\n\
                            │ │   │   │\n\
                            │.│. .│. .│\n\
                            └─┴───┴───┘\n\
                            ";

        let w = World::build_from_str(source_world).unwrap();

        let old_state = State::build(&w, (2, 3), Some('R'), 'B').unwrap();
        let new_state = State::build(&w, (2, 2), None, 'B').unwrap();

        let mut learner = EffectLearner::new();
        let test_state = State::build(&w, (3, 3), Some('Y'), 'B').unwrap();

        let predicted_0 = learner
            .predict(Attribute::Passenger, &w, &test_state)
            .unwrap();
        assert_eq!(predicted_0, None);

        learner.apply_experience(&Effect::generate_effects(
            Attribute::Passenger,
            &w,
            &old_state,
            &new_state,
        ));

        let expected_1 = Some(State::build(&w, (3, 3), None, 'B').unwrap());
        let predicted_1 = learner
            .predict(Attribute::Passenger, &w, &test_state)
            .unwrap();
        assert_eq!(predicted_1, expected_1);
    }

    #[test]
    fn learns_passenger_dropoff() {
        let source_world = "\
                            ┌───┬─────┐\n\
                            │R .│. . .│\n\
                            │   │     │\n\
                            │. .│G . .│\n\
                            │         │\n\
                            │. . . . .│\n\
                            │         │\n\
                            │.│Y .│B .│\n\
                            │ │   │   │\n\
                            │.│. .│. .│\n\
                            └─┴───┴───┘\n\
                            ";

        let w = World::build_from_str(source_world).unwrap();

        let old_state = State::build(&w, (2, 3), None, 'B').unwrap();
        let new_state = State::build(&w, (2, 2), Some('R'), 'B').unwrap();

        let mut learner = EffectLearner::new();
        let test_state = State::build(&w, (3, 3), None, 'B').unwrap();

        let predicted_0 = learner
            .predict(Attribute::Passenger, &w, &test_state)
            .unwrap();
        assert_eq!(predicted_0, None);

        learner.apply_experience(&Effect::generate_effects(
            Attribute::Passenger,
            &w,
            &old_state,
            &new_state,
        ));

        let expected_1 = Some(State::build(&w, (3, 3), Some('R'), 'B').unwrap());
        let predicted_1 = learner
            .predict(Attribute::Passenger, &w, &test_state)
            .unwrap();
        assert_eq!(predicted_1, expected_1);
    }
}
