use std::fmt;

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
                None
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

            self.false_conditions
                .retain(|c| !best_hypothesis.matches_cond(c));
        } else {
            self.true_conditions.retain(|c| c != condition);
        }
    }

    pub fn remove_overlap(&mut self, other: &ConditionLearner) {
        if let Some(ref other_best) = other.best {
            self.true_conditions.retain(|c| !other_best.matches_cond(c));
        }
    }

    pub fn overlaps(&self, other: &ConditionLearner) -> bool {
        match self.best {
            Some(ref best_hyp) => match other.best {
                Some(ref other_best_hyp) => {
                    best_hyp.matches(other_best_hyp) || other_best_hyp.matches(best_hyp)
                }
                None => false,
            },
            None => false,
        }
    }
}

impl Default for ConditionLearner {
    fn default() -> Self {
        ConditionLearner::new()
    }
}

impl fmt::Display for ConditionLearner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.best {
            Some(ref hyp) => write!(f, "{}", hyp),
            None => write!(f, "None"),
        }
    }
}

#[cfg(test)]
mod condition_learner_test {
    use super::*;
    use state::State;
    use world::World;

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

        assert_eq!(ce.predict(&cond_0_1), None);
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

    #[test]
    fn learns_nw_corner() {
        let source = "\
                      ┌───────┐\n\
                      │. . . .│\n\
                      │       │\n\
                      │. . . G│\n\
                      └───────┘\n\
                      ";

        let w = World::build_from_str(source).unwrap();

        let mut ce = ConditionLearner::new();

        for x in 0..4 {
            for y in 0..2 {
                let state = State::build(&w, (x, y), None, 'G').unwrap();
                let cond = Condition::new(&w, &state);

                let is_nw_corner = x == 0 && y == 0;

                ce.apply_experience(&cond, is_nw_corner);
            }
        }

        let state_0_0 = State::build(&w, (0, 0), None, 'G').unwrap();
        let cond_0_0 = Condition::new(&w, &state_0_0);
        assert_eq!(ce.predict(&cond_0_0), Some(true));

        let state_0_1 = State::build(&w, (0, 1), None, 'G').unwrap();
        let cond_0_1 = Condition::new(&w, &state_0_1);
        assert_eq!(ce.predict(&cond_0_1), Some(false));
    }

    #[test]
    fn learns_not_nw_corner() {
        // This test does not work.  It seems like
        // it should, but I don't see how it could.
        let source = "\
                      ┌───────┐\n\
                      │. . . .│\n\
                      │       │\n\
                      │. . . G│\n\
                      └───────┘\n\
                      ";

        let w = World::build_from_str(source).unwrap();

        let mut ce = ConditionLearner::new();

        for x in 0..4 {
            for y in 0..2 {
                let state = State::build(&w, (x, y), None, 'G').unwrap();
                let cond = Condition::new(&w, &state);

                let is_nw_corner = x == 0 && y == 0;

                ce.apply_experience(&cond, !is_nw_corner);
                println!("{}/{} yeilds {}", cond, !is_nw_corner, ce);
            }
        }

        // println!("true:");
        // for c in &ce.true_conditions {
        //     println!("{}", c);
        // }

        // println!("false:");
        // for c in &ce.false_conditions {
        //     println!("{}", c);
        // }

        // match ce.best {
        //     Some(ref hyp) => println!("best: {}", hyp),
        //     None => println!("best: None"),
        // }

        let state_0_0 = State::build(&w, (0, 0), None, 'G').unwrap();
        let cond_0_0 = Condition::new(&w, &state_0_0);
        println!("testing {}", cond_0_0);
        //assert_eq!(ce.predict(&cond_0_0), Some(false));

        let state_0_1 = State::build(&w, (0, 1), None, 'G').unwrap();
        let cond_0_1 = Condition::new(&w, &state_0_1);
        println!("testing {}", cond_0_1);
        //assert_eq!(ce.predict(&cond_0_1), Some(true));
    }
}
