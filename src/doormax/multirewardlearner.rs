use std::fmt;

use crate::doormax::condition::Condition;
use crate::doormax::condition_learner::ConditionLearner;

use crate::actions::Actions;
use crate::state::State;
use crate::world::World;

#[derive(Debug, Clone)]
pub struct RewardLearner {
    condition_rewards: Vec<(ConditionLearner, f64)>,
    error_delta: f64,
}

impl RewardLearner {
    pub fn new(error_delta: f64) -> Self {
        RewardLearner {
            condition_rewards: Vec::new(),
            error_delta,
        }
    }

    pub fn predict(&self, condition: &Condition) -> Option<f64> {
        let mut full_result: Option<f64> = None;

        for &(ref condition_learner, learned_reward) in &self.condition_rewards {
            let matches_condition = condition_learner.predict(condition);
            match matches_condition {
                None => {
                    return None;
                }
                Some(false) => (),
                Some(true) => {
                    if let Some(full_result) = full_result {
                        if !approx_eq!(f64, full_result, learned_reward, ulps = 2) {
                            // Conflicting result
                            // This should not be possible for rewards
                            // as they have only one effect.
                            return None;
                        }
                    } else {
                        full_result = Some(learned_reward);
                    }
                }
            };
        }

        // Should this return rmax instead of None?
        full_result
    }

    pub fn apply_experience(&mut self, condition: &Condition, reward: f64) {
        let mut found_entry = false;
        for &mut (ref mut condition_learner, learned_reward) in &mut self.condition_rewards {
            if approx_eq!(f64, reward, learned_reward, ulps = 2) {
                condition_learner.apply_experience(condition, true);
                found_entry = true;
            } else {
                condition_learner.apply_experience(condition, false);
            }
        }

        if !found_entry {
            let mut condition_learner = ConditionLearner::new();
            condition_learner.apply_experience(condition, true);

            for &(ref other_condition_learner, _) in &self.condition_rewards {
                condition_learner.remove_overlap(other_condition_learner);
            }

            // check for overlaps and remove old conditions if they exist.
            let mut has_conflict = false;

            for &(ref other_condition_learner, _other_reward) in &self.condition_rewards {
                if condition_learner.overlaps(other_condition_learner) {
                    // println!(
                    //     "Conflict with new condition {} => {} overlaps {} => {}",
                    //     condition_learner, reward, other_condition_learner, _other_reward
                    // );
                    has_conflict = true;
                    break;
                }
            }

            if has_conflict {
                self.condition_rewards = Vec::new();
            }

            // Now add our new condition_learner.
            self.condition_rewards.push((condition_learner, reward));
        } else {
            // Check for overlapping conditions.
            if !self.condition_rewards.is_empty() {
                let mut has_conflict = false;

                for i in 0..(self.condition_rewards.len() - 1) {
                    let &(ref condition_learner, _) = &self.condition_rewards[i];

                    for j in (i + 1)..self.condition_rewards.len() {
                        let &(ref other_condition_learner, _other_reward) =
                            &self.condition_rewards[j];

                        if condition_learner.overlaps(other_condition_learner) {
                            // println!(
                            //     "Conflict with existing condition {} => {} overlaps {} => {}",
                            //     condition_learner, reward, other_condition_learner, _other_reward
                            // );
                            has_conflict = true;
                            break;
                        }
                    }
                }

                if has_conflict {
                    self.condition_rewards = Vec::new();
                }
            }
        }
    }
}

impl fmt::Display for RewardLearner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CL(")?;
        let mut leader = " ";
        for &(ref condition_learner, learned_reward) in &self.condition_rewards {
            write!(f, "{}{} => {}", leader, condition_learner, learned_reward)?;
            leader = ", ";
        }
        write!(f, " )")
    }
}

#[derive(Debug, Clone)]
pub struct MultiRewardLearner {
    reward_learners: [RewardLearner; Actions::NUM_ELEMENTS],
}

impl MultiRewardLearner {
    pub fn new(error_delta: f64) -> Self {
        let reward_learners = [
            RewardLearner::new(error_delta),
            RewardLearner::new(error_delta),
            RewardLearner::new(error_delta),
            RewardLearner::new(error_delta),
            RewardLearner::new(error_delta),
            RewardLearner::new(error_delta),
        ];

        MultiRewardLearner { reward_learners }
    }

    pub fn predict(&self, world: &World, state: &State, action: Actions) -> Option<f64> {
        let condition = Condition::new(world, state);
        let action_index = action.to_index();

        self.reward_learners[action_index].predict(&condition)
    }

    pub fn apply_experience(&mut self, world: &World, state: &State, action: Actions, reward: f64) {
        let condition = Condition::new(world, state);
        let action_index = action.to_index();

        self.reward_learners[action_index].apply_experience(&condition, reward);

        // if action == Actions::DropOff {
        //     println!(
        //         "Applied experience, {} => {}, now {}",
        //         condition, reward, self.reward_learners[action_index]
        //     );
        // }
    }
}

impl fmt::Display for MultiRewardLearner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "reward:")?;
        for action_index in 0..Actions::NUM_ELEMENTS {
            let action = Actions::from_index(action_index).unwrap();
            writeln!(f, "{} - {}", action, self.reward_learners[action_index])?;
        }
        writeln!(f)?;
        Ok(())
    }
}

#[cfg(test)]
mod multirewardlearner_test {
    use super::*;
    use crate::actions::Actions;
    use crate::world::Costs;

    #[test]
    fn learns_pickup() {
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

        let costs = Costs::default();
        let w = World::build_from_str(source_world, costs).unwrap();

        let off_passenger = State::build(&w, (0, 1), Some('R'), 'B').unwrap();
        let (off_passenger_reward, _) = off_passenger.apply_action(&w, Actions::PickUp);
        assert!(approx_eq!(
            f64,
            off_passenger_reward,
            costs.miss_pickup,
            ulps = 1
        ));

        let mut learner = MultiRewardLearner::new(1.0e-6);

        assert_eq!(learner.predict(&w, &off_passenger, Actions::PickUp), None);

        learner.apply_experience(&w, &off_passenger, Actions::PickUp, off_passenger_reward);

        assert_eq!(
            learner.predict(&w, &off_passenger, Actions::PickUp),
            Some(off_passenger_reward)
        );

        let on_passenger = State::build(&w, (0, 0), Some('R'), 'B').unwrap();
        let (on_passenger_reward, _) = on_passenger.apply_action(&w, Actions::PickUp);
        assert!(approx_eq!(f64, on_passenger_reward, 0.0, ulps = 1));

        assert_eq!(learner.predict(&w, &on_passenger, Actions::PickUp), None);
        assert_eq!(
            learner.predict(&w, &off_passenger, Actions::PickUp),
            Some(off_passenger_reward)
        );

        learner.apply_experience(&w, &on_passenger, Actions::PickUp, on_passenger_reward);

        assert_eq!(
            learner.predict(&w, &on_passenger, Actions::PickUp),
            Some(on_passenger_reward)
        );
        assert_eq!(
            learner.predict(&w, &off_passenger, Actions::PickUp),
            Some(off_passenger_reward)
        );
    }

    #[test]
    fn learns_dropoff() {
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

        let costs = Costs::default();
        let w = World::build_from_str(source_world, costs).unwrap();

        let no_passenger = State::build(&w, (3, 3), Some('R'), 'B').unwrap();
        let (no_passenger_reward, _) = no_passenger.apply_action(&w, Actions::DropOff);
        assert!(approx_eq!(
            f64,
            no_passenger_reward,
            costs.empty_dropoff,
            ulps = 1
        ));

        let mut learner = MultiRewardLearner::new(1.0e-6);

        assert_eq!(learner.predict(&w, &no_passenger, Actions::DropOff), None);

        learner.apply_experience(&w, &no_passenger, Actions::DropOff, no_passenger_reward);

        assert_eq!(
            learner.predict(&w, &no_passenger, Actions::DropOff),
            Some(no_passenger_reward)
        );

        let off_destination = State::build(&w, (1, 3), None, 'B').unwrap();
        let (off_destination_reward, _) = off_destination.apply_action(&w, Actions::DropOff);
        assert!(approx_eq!(
            f64,
            off_destination_reward,
            costs.miss_dropoff,
            ulps = 1
        ));

        assert_eq!(
            learner.predict(&w, &off_destination, Actions::DropOff),
            None
        );

        learner.apply_experience(
            &w,
            &off_destination,
            Actions::DropOff,
            off_destination_reward,
        );

        assert_eq!(
            learner.predict(&w, &off_destination, Actions::DropOff),
            Some(off_destination_reward)
        );
        assert_eq!(
            learner.predict(&w, &no_passenger, Actions::DropOff),
            Some(no_passenger_reward)
        );

        let on_destination = State::build(&w, (3, 3), None, 'B').unwrap();
        let (on_destination_reward, _) = on_destination.apply_action(&w, Actions::DropOff);
        assert!(approx_eq!(f64, on_destination_reward, 0.0, ulps = 1));

        // This fails if miss_dropoff  and empty_dropoff are both -10. It will predict Some(-10)
        // because no_passenger and off_destination states have taught that those 2 conditions are **
        // for that effect.  This is what Diuk is talking about when he says disjunctions cannot be learned.
        // assert_eq!(learner.predict(&w, &on_destination, Actions::DropOff), None);

        learner.apply_experience(&w, &on_destination, Actions::DropOff, on_destination_reward);

        // off_destination and no_passenger will now be None because they were removed
        // as conflicts.
        assert_eq!(
            learner.predict(&w, &on_destination, Actions::DropOff),
            Some(on_destination_reward)
        );
        assert_eq!(
            learner.predict(&w, &off_destination, Actions::DropOff),
            Some(off_destination_reward)
        );
        assert_eq!(
            learner.predict(&w, &no_passenger, Actions::DropOff),
            Some(no_passenger_reward)
        );
    }
}
