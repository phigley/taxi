
use std::f64;
use std::cmp;

use rand::Rng;
use float_cmp::ApproxOrdUlps;

use state::State;
use actions::Actions;
use world::World;

use runner::{Runner, Attempt};
use state_indexer::StateIndexer;

#[derive(Debug, Clone)]
struct TransitionEntry {
    destination_count: Vec<f64>,
    count: f64,
}

impl TransitionEntry {
    fn new(num_states: usize) -> TransitionEntry {

        TransitionEntry {
            destination_count: vec![0.0; num_states],
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
pub struct FactoredRMax {
    state_indexer: StateIndexer,
    rmax: f64,

    transition_table: Vec<TransitionEntry>,
    reward_table: Vec<RewardEntry>,

    value_table: Vec<f64>,

    gamma: f64,
    error_delta: f64,
    known_count: f64,
}

impl FactoredRMax {
    pub fn new(world: &World, gamma: f64, known_count: f64, error_delta: f64) -> FactoredRMax {

        let state_indexer = StateIndexer::new(world);
        let num_states = state_indexer.num_states();
        let value_table = vec![0.0; num_states];

        let transition_table =
            vec![TransitionEntry::new(num_states); num_states * Actions::NUM_ELEMENTS];

        let reward_table = vec![RewardEntry::default(); num_states * Actions::NUM_ELEMENTS];

        FactoredRMax {
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
            transition_entry.destination_count[next_state_index] += 1.0;
        }

        let reward_entry = &mut self.reward_table[state_action_index];
        if reward_entry.count < self.known_count {
            reward_entry.count += 1.0;

            let delta = reward - reward_entry.mean;

            reward_entry.mean += delta / reward_entry.count;
        }
    }


    fn predict_transition_reward(
        &self,
        state_index: usize,
        action_index: usize,
        next_state_index: usize,
    ) -> (f64, f64) {

        let state_action_index = state_index * Actions::NUM_ELEMENTS + action_index;

        let transition_entry = &self.transition_table[state_action_index];
        let reward_entry = &self.reward_table[state_action_index];

        if transition_entry.count >= self.known_count && reward_entry.count >= self.known_count {
            let transition = transition_entry.destination_count[next_state_index] /
                transition_entry.count;
            let reward = reward_entry.mean;

            (transition, reward)

        } else {

            let transition = if state_index == next_state_index {
                1.0
            } else {
                0.0
            };

            (transition, self.rmax)
        }
    }

    fn measure_best_value(&self, state_index: usize) -> f64 {

        let mut best_value = -f64::MAX;

        for action_index in 0..Actions::NUM_ELEMENTS {
            let mut action_value = 0.0;

            for next_state_index in 0..self.state_indexer.num_states() {
                let (transition, reward) =
                    self.predict_transition_reward(state_index, action_index, next_state_index);
                action_value += transition *
                    (reward + self.gamma * self.value_table[next_state_index]);
            }

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
            let mut action_value = 0.0;

            for next_state_index in 0..self.state_indexer.num_states() {
                let (transition, reward) =
                    self.predict_transition_reward(state_index, action_index, next_state_index);
                action_value += transition *
                    (reward + self.gamma * self.value_table[next_state_index]);
            }

            match action_value.approx_cmp(&best_value, 2) {
                cmp::Ordering::Greater => {
                    best_value = action_value;
                    best_action_index = action_index;
                    num_found = 1;
                }
                cmp::Ordering::Equal => {
                    num_found += 1;

                    if 0 == rng.gen_range(0, num_found) {
                        best_action_index = action_index;
                    }
                }
                cmp::Ordering::Less => {}
            }
        }

        best_action_index
    }

    fn rebuild_value_table(&mut self) {

        let num_states = self.state_indexer.num_states();

        //self.value_table.iter_mut().for_each(|v| *v = 0.0);

        for _ in 0..10000 {
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


impl Runner for FactoredRMax {
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

                    let reward = state.apply_action(world, next_action);

                    if let Some(next_state_index) = self.state_indexer.get_index(world, &state) {
                        self.apply_experience(state_index, next_action, next_state_index, reward);
                    } else {
                        return None;
                    }

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
                    state.apply_action(world, next_action);
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
                    state.apply_action(world, next_action);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        state.at_destination()
    }

    fn report_training_result(&self, _world: &World) {}
}
