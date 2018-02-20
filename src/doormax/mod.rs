mod term;
mod condition;
mod hypothesis;
mod effect;
mod condition_learner;
mod effect_learner;
mod mcelearner;

use std::f64;
use std::cmp;

use rand::Rng;
// use rand::Isaac64Rng;
use float_cmp::ApproxOrdUlps;

use state;
use state::{State, StateIterator};
use world::World;
use state_indexer::StateIndexer;
use actions::Actions;

use self::mcelearner::MCELearner;

use runner::{Attempt, Runner};

#[derive(Debug, Clone, Copy, Default)]
struct RewardEntry {
    mean: f64,
    count: f64,
}

#[derive(Debug, Clone)]
pub struct DoorMax {
    state_indexer: StateIndexer,
    rmax: f64,

    mcelearner: MCELearner,

    reward_table: Vec<RewardEntry>,
    known_reward_count: f64,

    value_table: Vec<f64>,

    gamma: f64,
    error_delta: f64,
}

impl DoorMax {
    pub fn new(world: &World, gamma: f64, known_reward_count: f64, error_delta: f64) -> Self {
        let state_indexer = StateIndexer::new(world);
        let num_states = state_indexer.num_states();
        let value_table = vec![0.0; num_states];

        let reward_table = vec![RewardEntry::default(); num_states * Actions::NUM_ELEMENTS];

        DoorMax {
            state_indexer,
            rmax: world.max_reward(),

            mcelearner: MCELearner::new(),

            reward_table,
            known_reward_count,

            value_table,

            gamma,
            error_delta,
        }
    }

    fn apply_experience(
        &mut self,
        world: &World,
        state: &State,
        action: Actions,
        new_state: &State,
        reward: f64,
    ) {
        self.mcelearner
            .apply_experience(world, state, action, new_state);

        let state_index = self.state_indexer.get_index(world, state).unwrap();
        let action_index = action.to_index();
        let state_action_index = state_index * Actions::NUM_ELEMENTS + action_index;
        let reward_entry = &mut self.reward_table[state_action_index];
        if reward_entry.count < self.known_reward_count {
            reward_entry.count += 1.0;

            let delta = reward - reward_entry.mean;
            reward_entry.mean += delta / reward_entry.count;
        }
    }

    fn measure_reward(&self, state_index: usize, action: Actions) -> f64 {
        let state_action_index = state_index * Actions::NUM_ELEMENTS + action.to_index();

        let reward_entry = &self.reward_table[state_action_index];

        if reward_entry.count >= self.known_reward_count {
            reward_entry.mean
        } else {
            self.rmax
        }
    }

    fn measure_value(
        &self,
        world: &World,
        state: &State,
        action: Actions,
    ) -> Result<f64, state::Error> {
        let state_index = self.state_indexer.get_index(world, state).unwrap();

        let mut action_value = self.measure_reward(state_index, action);

        if let Some(next_state) = self.mcelearner.predict(world, state, action)? {
            let next_state_index = self.state_indexer.get_index(world, &next_state).unwrap();
            action_value += self.gamma * self.value_table[next_state_index];
        } else {
            // Assume we will stay in our current state.
            action_value += self.gamma * self.value_table[state_index];
        }

        Ok(action_value)
    }

    fn measure_best_value(&self, world: &World, state: &State) -> Result<f64, state::Error> {
        let mut best_value = -f64::MAX;

        for action_index in 0..Actions::NUM_ELEMENTS {
            let action = Actions::from_index(action_index).unwrap();
            let action_value = self.measure_value(world, state, action)?;

            if action_value > best_value {
                best_value = action_value;
            }
        }

        Ok(best_value)
    }

    fn select_best_action<R: Rng>(
        &self,
        world: &World,
        state: &State,
        rng: &mut R,
    ) -> Result<Option<Actions>, state::Error> {
        let mut best_value = -f64::MAX;
        let mut best_action = None;
        let mut num_found = 0;

        for action_index in 0..Actions::NUM_ELEMENTS {
            let action = Actions::from_index(action_index).unwrap();
            let action_value = self.measure_value(world, state, action)?;

            match action_value.approx_cmp(&best_value, 2) {
                cmp::Ordering::Greater => {
                    best_value = action_value;
                    best_action = Some(action);
                    num_found = 1;
                }
                cmp::Ordering::Equal => {
                    num_found += 1;

                    if 0 == rng.gen_range(0, num_found) {
                        best_action = Some(action);
                    }
                }
                cmp::Ordering::Less => {}
            }
        }

        Ok(best_action)
    }

    fn rebuild_value_table(&mut self, world: &World) -> Result<(), state::Error> {
        for _ in 0..10_000 {
            let mut error = 0.0;

            for state in StateIterator::new(world) {
                if state.at_destination() {
                    continue;
                }

                let state_index = self.state_indexer.get_index(world, &state).unwrap();

                let old_value = self.value_table[state_index];

                let new_value = self.measure_best_value(world, &state)?;

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

        Ok(())
    }

    fn learn<R: Rng>(
        &mut self,
        world: &World,
        mut state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Result<Option<usize>, state::Error> {
        for step in 0..max_steps {
            if state.at_destination() {
                return Ok(Some(step));
            }

            self.rebuild_value_table(world)?;

            if let Some(next_action) = self.select_best_action(world, &state, rng)? {
                let (reward, next_state) = state.apply_action(world, next_action);

                self.apply_experience(world, &state, next_action, &next_state, reward);
                state = next_state;
            } else {
                return Ok(None);
            }
        }

        if state.at_destination() {
            Ok(Some(max_steps))
        } else {
            Ok(None)
        }
    }

    fn attempt<R: Rng>(
        &self,
        world: &World,
        mut state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Result<Attempt, state::Error> {
        let mut attempt = Attempt::new(state, max_steps);

        for _ in 0..max_steps {
            if state.at_destination() {
                break;
            }

            if let Some(next_action) = self.select_best_action(world, &state, rng)? {
                attempt.step(next_action);
                let (_, next_state) = state.apply_action(world, next_action);
                state = next_state;
            } else {
                break;
            }
        }

        if state.at_destination() {
            attempt.succeeded()
        }

        Ok(attempt)
    }

    fn solves<R: Rng>(
        &self,
        world: &World,
        mut state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Result<bool, state::Error> {
        for _ in 0..max_steps {
            if state.at_destination() {
                return Ok(true);
            }

            if let Some(next_action) = self.select_best_action(world, &state, rng)? {
                let (_, next_state) = state.apply_action(world, next_action);
                state = next_state;
            } else {
                break;
            }
        }

        Ok(state.at_destination())
    }
}

impl Runner for DoorMax {
    fn learn<R: Rng>(
        &mut self,
        world: &World,
        state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Option<usize> {
        self.learn(world, state, max_steps, rng).unwrap()
    }

    fn attempt<R: Rng>(
        &self,
        world: &World,
        state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Attempt {
        self.attempt(world, state, max_steps, rng).unwrap()
    }

    fn solves<R: Rng>(&self, world: &World, state: State, max_steps: usize, rng: &mut R) -> bool {
        self.solves(world, state, max_steps, rng).unwrap()
    }

    fn report_training_result(&self, world: &World) {
        // let mut rng = Isaac64Rng::new_unseeded();

        // let num_states = self.state_indexer.num_states();
        // for state_index in 0..num_states {
        //     let state = self.state_indexer.get_state(world, state_index).unwrap();

        //     if !state.at_destination() {
        //         if let Some(next_action) = self.select_best_action(world, &state, &mut rng).unwrap()
        //         {
        //             println!("===================");
        //             println!("{}", state.display(world));
        //             println!("Best action: {}", next_action);

        //             if let Some(next_state) =
        //                 self.mcelearner.predict(world, &state, next_action).unwrap()
        //             {
        //                 println!("{}", next_state.display(world));
        //             }

        //             for action_index in 0..Actions::NUM_ELEMENTS {
        //                 let action = Actions::from_index(action_index).unwrap();

        //                 let reward = self.measure_reward(state_index, action);
        //                 let action_value = self.measure_value(world, &state, action).unwrap();
        //                 println!(
        //                     "{} - {} + {} = {}",
        //                     action,
        //                     reward,
        //                     action_value,
        //                     reward + action_value
        //                 );
        //             }
        //         }
        //     }
        // }
    }
}
