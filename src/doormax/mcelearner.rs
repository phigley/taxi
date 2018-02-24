use doormax::condition_learner::ConditionLearner;
use doormax::effect_learner::EffectLearner;
use doormax::effect;
use doormax::effect::{ChangePassenger, ChangeTaxiX, ChangeTaxiY, Effect};
use doormax::condition::Condition;

use world::World;
use state::State;
use actions::Actions;

#[derive(Debug, Clone)]
pub struct CELearner<E: Effect> {
    condition_learner: ConditionLearner,
    effect_learner: EffectLearner<E>,
}

impl<E: Effect> CELearner<E> {
    pub fn new() -> Self {
        CELearner {
            condition_learner: ConditionLearner::new(),
            effect_learner: EffectLearner::new(),
        }
    }

    pub fn predict(
        &self,
        world: &World,
        state: &State,
        condition: &Condition,
    ) -> Result<Option<State>, effect::Error> {
        let matches_condition = self.condition_learner.predict(condition);
        let result = match matches_condition {
            None => None,
            Some(false) => Some(*state),
            Some(true) => self.effect_learner.predict(world, state)?,
        };

        Ok(result)
    }

    pub fn apply_experience(&mut self, condition: &Condition, old_state: &State, new_state: &State)
    where
        E: Clone + PartialEq,
    {
        let effects = E::generate_effects(old_state, new_state);

        // If we were to enforce a maximum number of effects, we should check it here.
        // if effects.len() > 5 {
        //     *learner = CELearner::default();
        // }

        self.condition_learner
            .apply_experience(condition, !effects.is_empty());
        self.effect_learner.apply_experience(&effects);
    }
}

impl<E: Effect> Default for CELearner<E> {
    fn default() -> Self {
        CELearner::new()
    }
}

#[derive(Debug, Clone)]
pub struct MCELearner {
    taxi_x_learners: [CELearner<ChangeTaxiX>; Actions::NUM_ELEMENTS],
    taxi_y_learners: [CELearner<ChangeTaxiY>; Actions::NUM_ELEMENTS],
    passenger_learners: [CELearner<ChangePassenger>; Actions::NUM_ELEMENTS],
}

impl MCELearner {
    pub fn new() -> Self {
        MCELearner {
            taxi_x_learners: Default::default(),
            taxi_y_learners: Default::default(),
            passenger_learners: Default::default(),
        }
    }

    pub fn predict(
        &self,
        world: &World,
        state: &State,
        action: Actions,
    ) -> Result<Option<State>, effect::Error> {
        let condition = Condition::new(world, state);
        let action_index = action.to_index();

        if let Some(predicted_taxi_x) =
            self.taxi_x_learners[action_index].predict(world, state, &condition)?
        {
            if let Some(predicted_taxi_y) =
                self.taxi_y_learners[action_index].predict(world, state, &condition)?
            {
                if let Some(predicted_passenger) =
                    self.passenger_learners[action_index].predict(world, state, &condition)?
                {
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
        let action_index = action.to_index();

        self.taxi_x_learners[action_index].apply_experience(&condition, state, new_state);
        self.taxi_y_learners[action_index].apply_experience(&condition, state, new_state);
        self.passenger_learners[action_index].apply_experience(&condition, state, new_state);
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
        assert_eq!(predicted_0, Some(new_state));
    }
}
