use enum_map::EnumMap;

use doormax::condition_learner::ConditionLearner;
use doormax::effect_learner::EffectLearner;
use doormax::effect::{Attribute, Effect};
use doormax::condition::Condition;

use world::World;
use state;
use state::State;
use actions::Actions;

#[derive(Debug, Clone)]
pub struct CELearner {
    condition_learner: ConditionLearner,
    effect_learner: EffectLearner,
}

impl CELearner {
    pub fn new() -> Self {
        CELearner {
            condition_learner: ConditionLearner::new(),
            effect_learner: EffectLearner::new(),
        }
    }

    pub fn predict(
        &self,
        attribute: Attribute,
        world: &World,
        state: &State,
        condition: &Condition,
    ) -> Result<Option<State>, state::Error> {
        let matches_condition = self.condition_learner.predict(condition);
        let result = match matches_condition {
            None => None,
            Some(false) => Some(*state),
            Some(true) => self.effect_learner.predict(attribute, world, state)?,
        };

        Ok(result)
    }

    pub fn apply_experience(&mut self, condition: &Condition, effects: &[Effect]) {
        self.condition_learner
            .apply_experience(condition, !effects.is_empty());
        self.effect_learner.apply_experience(effects);
    }
}

impl Default for CELearner {
    fn default() -> Self {
        CELearner::new()
    }
}

#[derive(Debug, Clone)]
pub struct MCELearner {
    learners: [EnumMap<Attribute, CELearner>; Actions::NUM_ELEMENTS],
}

impl MCELearner {
    pub fn new() -> Self {
        MCELearner {
            learners: [
                EnumMap::default(),
                EnumMap::default(),
                EnumMap::default(),
                EnumMap::default(),
                EnumMap::default(),
                EnumMap::default(),
            ],
        }
    }

    pub fn predict(
        &self,
        world: &World,
        state: &State,
        action: Actions,
    ) -> Result<Option<State>, state::Error> {
        let condition = Condition::new(world, state);

        let action_learners = &self.learners[action.to_index()];

        if let Some(predicted_taxi_x) =
            action_learners[Attribute::TaxiX].predict(Attribute::TaxiX, world, state, &condition)?
        {
            if let Some(predicted_taxi_y) = action_learners[Attribute::TaxiY].predict(
                Attribute::TaxiY,
                world,
                state,
                &condition,
            )? {
                if let Some(predicted_passenger) = action_learners[Attribute::Passenger].predict(
                    Attribute::Passenger,
                    world,
                    state,
                    &condition,
                )? {
                    return Ok(Some(State::build(
                        world,
                        (predicted_taxi_x.get_taxi().x, predicted_taxi_y.get_taxi().y),
                        predicted_passenger.get_passenger(),
                        state.get_destination(),
                    )?));
                }
            }
        }

        Ok(None)
    }

    pub fn apply_experience(
        &mut self,
        world: &World,
        state: &State,
        action: Actions,
        new_state: &State,
    ) {
        let condition = Condition::new(world, state);

        let action_learners = &mut self.learners[action.to_index()];

        for (attribute, learner) in action_learners {
            // I'm not separating effects by type, is that important?
            // For Taxi problem, there is only one type per attribute.
            let effects = Effect::generate_effects(attribute, world, state, new_state);

            // If we were to enforce a maximum number of effects, we should check it here.
            // if effects.len() > 5 {
            //     *learner = CELearner::default();
            // }

            // How do we check for conflicting conditions?

            learner.apply_experience(&condition, &effects);
        }
    }
}

#[cfg(test)]
mod mcelearner_test {
    use super::*;
    use position::Position;

    #[test]
    fn learns_taxi_east() {
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
        let (_, new_state) = old_state.apply_action(&w, Actions::East);
        assert_eq!(*new_state.get_taxi(), Position::new(2, 3));

        let mut learner = MCELearner::new();
        learner.apply_experience(&w, &old_state, Actions::East, &new_state);

        let predicted_0 = learner.predict(&w, &old_state, Actions::East).unwrap();

        for (attribute, learner) in &learner.learners[Actions::East.to_index()] {
            println!("{:#?} - {:#?}", attribute, learner);
        }
        assert_eq!(predicted_0, Some(new_state));
    }
}
