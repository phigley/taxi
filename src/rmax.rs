use std::collections::HashMap;
use std::f64;

use rand::Rng;
use rand_pcg::Pcg64Mcg;

use crate::actions::Actions;
use crate::state::State;
use crate::world::World;

use crate::runner::{Attempt, Runner};
use crate::state_indexer::StateIndexer;

#[derive(Debug, Clone)]
struct TransitionEntry {
    destination_counts: HashMap<usize, f64>,
    count: f64,
}

impl TransitionEntry {
    fn new(maximum_count: usize) -> TransitionEntry {
        TransitionEntry {
            destination_counts: HashMap::with_capacity(maximum_count),
            count: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct RewardEntry {
    mean: f64,
    count: f64,
}

#[derive(Debug, Clone)]
pub struct RMax {
    state_indexer: StateIndexer,
    rmax: f64,

    transition_table: Vec<TransitionEntry>,
    reward_table: Vec<RewardEntry>,

    value_table: Vec<f64>,

    gamma: f64,
    error_delta: f64,
    known_count: f64,
}

impl RMax {
    pub fn new(world: &World, gamma: f64, known_count: f64, error_delta: f64) -> RMax {
        let state_indexer = StateIndexer::new(world);
        let num_states = state_indexer.num_states();
        let value_table = vec![0.0; num_states];

        let transition_table =
            vec![TransitionEntry::new(num_states); num_states * Actions::NUM_ELEMENTS];

        let reward_table = vec![RewardEntry::default(); num_states * Actions::NUM_ELEMENTS];

        RMax {
            state_indexer,
            rmax: world.max_reward(),

            transition_table,
            reward_table,

            value_table,

            gamma,
            known_count,
            error_delta,
        }
    }

    fn apply_experience(
        &mut self,
        state_index: usize,
        action: Actions,
        next_state_index: usize,
        reward: f64,
    ) {
        let action_index = action.to_index();

        let state_action_index = state_index * Actions::NUM_ELEMENTS + action_index;

        let transition_entry = &mut self.transition_table[state_action_index];

        if transition_entry.count < self.known_count {
            transition_entry.count += 1.0;
            let destination_count = transition_entry
                .destination_counts
                .entry(next_state_index)
                .or_insert(0.0);
            *destination_count += 1.0;
        }

        let reward_entry = &mut self.reward_table[state_action_index];
        if reward_entry.count < self.known_count {
            reward_entry.count += 1.0;

            let delta = reward - reward_entry.mean;

            reward_entry.mean += delta / reward_entry.count;
        }
    }

    fn measure_value(&self, state_index: usize, action_index: usize) -> f64 {
        let state_action_index = state_index * Actions::NUM_ELEMENTS + action_index;

        let transition_entry = &self.transition_table[state_action_index];
        let reward_entry = &self.reward_table[state_action_index];

        if reward_entry.count >= self.known_count && transition_entry.count >= self.known_count {
            let mut action_value = reward_entry.mean;

            for (next_state_index, transition_count) in &transition_entry.destination_counts {
                let transition = transition_count / self.known_count;

                action_value += transition * self.gamma * self.value_table[*next_state_index];
            }

            action_value
        } else {
            // Return maximum reward and the value of staying in this current state.
            self.rmax + self.gamma * self.value_table[state_index]
        }
    }

    fn measure_best_value(&self, state_index: usize) -> f64 {
        let mut best_value = -f64::MAX;

        for action_index in 0..Actions::NUM_ELEMENTS {
            let action_value = self.measure_value(state_index, action_index);

            if action_value > best_value {
                best_value = action_value;
            }
        }

        best_value
    }

    fn determine_best_action_index<R: Rng>(&self, state_index: usize, rng: &mut R) -> usize {
        let mut best_value = -f64::MAX;
        let mut best_action_index = Actions::NUM_ELEMENTS;
        let mut num_found = 0;

        for action_index in 0..Actions::NUM_ELEMENTS {
            let action_value = self.measure_value(state_index, action_index);

            if approx_eq!(f64, action_value, best_value, ulps = 2) {
                num_found += 1;

                if 0 == rng.gen_range(0, num_found) {
                    best_action_index = action_index;
                }
            } else if action_value > best_value {
                best_value = action_value;
                best_action_index = action_index;
                num_found = 1;
            }
        }

        best_action_index
    }

    fn rebuild_value_table(&mut self) {
        let num_states = self.state_indexer.num_states();

        for _ in 0..10_000 {
            let mut error = 0.0;

            for state_index in 0..num_states {
                let old_value = self.value_table[state_index];

                let new_value = self.measure_best_value(state_index);

                self.value_table[state_index] = new_value;

                let state_error = (new_value - old_value).abs();
                if state_error > error {
                    error = state_error;
                }
            }

            if error < self.error_delta {
                break;
            }
        }
    }

    fn select_best_action<R: Rng>(&self, state_index: usize, rng: &mut R) -> Option<Actions> {
        let action_index = self.determine_best_action_index(state_index, rng);
        Actions::from_index(action_index)
    }
}

impl Runner for RMax {
    fn learn<R: Rng>(
        &mut self,
        world: &World,
        mut state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Option<usize> {
        for step in 0..max_steps {
            if state.at_destination() {
                return Some(step);
            }

            self.rebuild_value_table();

            if let Some(state_index) = self.state_indexer.get_index(world, &state) {
                if let Some(next_action) = self.select_best_action(state_index, rng) {
                    let (reward, next_state) = state.apply_action(world, next_action);

                    if let Some(next_state_index) = self.state_indexer.get_index(world, &next_state)
                    {
                        self.apply_experience(state_index, next_action, next_state_index, reward);
                    } else {
                        return None;
                    }

                    state = next_state;
                } else {
                    return None;
                }
            }
        }

        if state.at_destination() {
            Some(max_steps)
        } else {
            None
        }
    }

    fn attempt<R: Rng>(
        &self,
        world: &World,
        mut state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Attempt {
        let mut attempt = Attempt::new(state, max_steps);

        for _ in 0..max_steps {
            if state.at_destination() {
                break;
            }

            if let Some(state_index) = self.state_indexer.get_index(world, &state) {
                if let Some(next_action) = self.select_best_action(state_index, rng) {
                    attempt.step(next_action);
                    let (_, next_state) = state.apply_action(world, next_action);
                    state = next_state;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if state.at_destination() {
            attempt.succeeded()
        }

        attempt
    }

    fn solves<R: Rng>(
        &self,
        world: &World,
        mut state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> bool {
        for _ in 0..max_steps {
            if state.at_destination() {
                return true;
            }

            if let Some(state_index) = self.state_indexer.get_index(world, &state) {
                if let Some(next_action) = self.select_best_action(state_index, rng) {
                    let (_, next_state) = state.apply_action(world, next_action);
                    state = next_state;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        state.at_destination()
    }

    fn report_training_result(&self, world: &World, _steps: Option<usize>) {
        let mut rng = Pcg64Mcg::new(0xcafe_f00d_d15e_a5e5);

        let num_states = self.state_indexer.num_states();
        for state_index in 0..num_states {
            let state = self.state_indexer.get_state(world, state_index).unwrap();

            if !state.at_destination() {
                if let Some(next_action) = self.select_best_action(state_index, &mut rng) {
                    println!("===================");
                    println!("{}", state.display(world));
                    println!("Best action: {}", next_action);

                    for action_index in 0..Actions::NUM_ELEMENTS {
                        let action = Actions::from_index(action_index).unwrap();

                        let action_value = self.measure_value(state_index, action_index);

                        println!("{} - {}", action, action_value);
                    }
                }
            }
        }
    }
}
