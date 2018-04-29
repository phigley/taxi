use std::fmt;

use doormax::condition::Condition;
use doormax::condition_learner::ConditionLearner;

use actions::Actions;
use state::State;
use world::World;


#[derive(Debug, Clone)]
pub struct RewardLearner {
    condition_rewards: Vec<(ConditionLearner, f64)>,
}

impl RewardLearner {
    pub fn new() -> Self {
        RewardLearner {
            condition_rewards: Vec::new(),
        }
    }

    pub fn predict(
        &self,
        condition: &Condition,
    ) -> Option<f64> {
        let mut full_result = None;

        for &(ref condition_learner, learned_reward) in &self.condition_rewards {
            let matches_condition = condition_learner.predict(condition);
            match matches_condition {
                None => {
                    return None;
                }
                Some(false) => (),
                Some(true) => {
                    if let Some(full_result) = full_result {
                        if full_result != learned_reward {
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

    pub fn apply_experience(&mut self, condition: &Condition, reward: f64)
    {
        let mut found_entry = false;
        for &mut (ref mut condition_learner, learned_reward) in &mut self.condition_rewards {
            if reward == learned_reward {
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

            for &(ref other_condition_learner, _) in &self.condition_rewards {
                if condition_learner.overlaps(other_condition_learner) {
                    has_conflict = true;
                    break;
                }
            }

            if has_conflict {
                self.condition_rewards = Vec::new();
            }

            // Now add our new condition_learner.
            self.condition_rewards
                .push((condition_learner, reward));

        } else {

            // Check for overlapping conditions.
            if !self.condition_rewards.is_empty() {
                let mut has_conflict = false;

                for i in 0..(self.condition_rewards.len() - 1) {
                    let &(ref condition_learner, _) = &self.condition_rewards[i];

                    for j in (i + 1)..self.condition_rewards.len() {
                        let &(ref other_condition_learner, _) = &self.condition_rewards[j];

                        if condition_learner.overlaps(other_condition_learner) {
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

impl Default for RewardLearner {
    fn default() -> Self {
        RewardLearner::new()
    }
}

impl fmt::Display for RewardLearner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    pub fn new() -> Self {
        MultiRewardLearner {
            reward_learners: Default::default(),
        }
    }

    pub fn predict(
        &self,
        world: &World,
        state: &State,
        action: Actions,
    ) -> Option<f64> {
        let condition = Condition::new(world, state);
        let action_index = action.to_index();

        self.reward_learners[action_index].predict(&condition)
    }

    pub fn apply_experience(
        &mut self,
        world: &World,
        state: &State,
        action: Actions,
        reward: f64,
    ) {
        let condition = Condition::new(world, state);
        let action_index = action.to_index();

        self.reward_learners[action_index].apply_experience(&condition, reward);
    }
}

impl fmt::Display for MultiRewardLearner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "reward:\n")?;
        for action_index in 0..Actions::NUM_ELEMENTS {
            let action = Actions::from_index(action_index).unwrap();
            write!(f, "{} - {}\n", action, self.reward_learners[action_index])?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}