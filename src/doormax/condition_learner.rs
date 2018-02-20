use doormax::condition::Condition;
use doormax::hypothesis::Hypothesis;

#[derive(Debug, Clone)]
pub struct ConditionLearner {
    true_conditions: Vec<Condition>,
    false_conditions: Vec<Condition>,
    best: Option<Hypothesis>,
}

impl ConditionLearner {
    pub fn new() -> Self {
        let true_conditions = Condition::enumerate_all();
        let false_conditions = Condition::enumerate_all();

        ConditionLearner {
            true_conditions,
            false_conditions,

            best: None,
        }
    }

    pub fn predict(&self, condition: &Condition) -> Option<bool> {
        let has_failure = self.false_conditions.iter().any(|c| c == condition);
        let has_true = self.true_conditions.iter().any(|c| c == condition);

        match self.best {
            None => if !has_failure {
                Some(true)
            } else {
                Some(false)
            },

            Some(ref best_hypothesis) => if best_hypothesis.matches_cond(condition) {
                if !has_failure {
                    Some(true)
                } else {
                    None
                }
            } else {
                if has_failure && !has_true {
                    Some(false)
                } else {
                    None
                }
            },
        }
    }

    pub fn apply_experience(&mut self, condition: &Condition, truth: bool) {
        if truth {
            match self.best {
                None => {
                    self.best = Some(Hypothesis::from(condition.clone()));
                }
                Some(ref mut best_hypothesis) => {
                    *best_hypothesis = best_hypothesis.combine_cond(condition);
                }
            }

            let best_hypothesis = self.best.as_ref().unwrap();

            for i in (0..self.false_conditions.len()).rev() {
                if best_hypothesis.matches_cond(&self.false_conditions[i]) {
                    self.false_conditions.swap_remove(i);
                }
            }
        } else {
            for i in (0..self.true_conditions.len()).rev() {
                if self.true_conditions[i] == *condition {
                    self.true_conditions.swap_remove(i);
                }
            }
        }
    }
}

impl Default for ConditionLearner {
    fn default() -> Self {
        ConditionLearner::new()
    }
}

#[cfg(test)]
mod condition_learner_test {
    use super::*;
    use world::World;
    use state::State;

    #[test]
    fn learns_north() {
        let source = "\
                      ┌───────┐\n\
                      │. . . .│\n\
                      │       │\n\
                      │. . . G│\n\
                      └───────┘\n\
                      ";

        let w = World::build_from_str(source).unwrap();

        let mut ce = ConditionLearner::new();

        let state_0_0 = State::build(&w, (0, 0), None, 'G').unwrap();
        let cond_0_0 = Condition::new(&w, &state_0_0);

        let state_0_1 = State::build(&w, (0, 1), None, 'G').unwrap();
        let cond_0_1 = Condition::new(&w, &state_0_1);

        let state_1_0 = State::build(&w, (1, 0), None, 'G').unwrap();
        let cond_1_0 = Condition::new(&w, &state_1_0);

        let state_1_1 = State::build(&w, (1, 1), None, 'G').unwrap();
        let cond_1_1 = Condition::new(&w, &state_1_1);

        let state_2_0 = State::build(&w, (2, 0), None, 'G').unwrap();
        let cond_2_0 = Condition::new(&w, &state_2_0);

        let state_2_1 = State::build(&w, (2, 1), None, 'G').unwrap();
        let cond_2_1 = Condition::new(&w, &state_2_1);

        // Notice how the prediction changes.  We need to predict
        // false so that actions that have no effect will learn from
        // the falses.
        assert_eq!(ce.predict(&cond_0_1), Some(false));
        ce.apply_experience(&cond_0_1, true);
        assert_eq!(ce.predict(&cond_0_1), Some(true));

        // This has a new condition, so it should be
        // uncertain.
        assert_eq!(ce.predict(&cond_1_1), None);
        ce.apply_experience(&cond_1_1, true);
        assert_eq!(ce.predict(&cond_1_1), Some(true));

        // Note that this one will predict Some(false) if
        // we are not checking for un-observed positive conditions.
        assert_eq!(ce.predict(&cond_0_0), None);
        ce.apply_experience(&cond_0_0, false);
        assert_eq!(ce.predict(&cond_0_0), Some(false));

        assert_eq!(ce.predict(&cond_1_0), None);
        ce.apply_experience(&cond_1_0, false);
        assert_eq!(ce.predict(&cond_1_0), Some(false));

        assert_eq!(ce.predict(&cond_2_0), Some(false));
        assert_eq!(ce.predict(&cond_2_1), Some(true));
    }
}
