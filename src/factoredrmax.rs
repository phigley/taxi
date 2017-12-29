use std::f64;
use std::cmp;

use rand::Rng;
//use rand::Isaac64Rng;
use float_cmp::ApproxOrdUlps;

use state::State;
use actions::Actions;
use world::World;

use runner::{Attempt, Runner};
use state_indexer::StateIndexer;

#[derive(Debug, Clone)]
struct Transitions {
    parent_index_starts: [usize; 4 * Actions::NUM_ELEMENTS],
    occurences: Vec<f64>, // parent_index_start + parent_index

    transition_starts: Vec<usize>, // parent_index_start + parent_index
    transitions: Vec<f64>,         // transition_start + destination_value

    known_count: f64,
}

impl Transitions {
    fn new(world: &World, known_count: f64) -> Transitions {
        let num_x_states = world.width as usize;
        let num_y_states = world.height as usize;
        let num_destination_states = world.num_fixed_positions();
        let num_passenger_states = num_destination_states + 1;

        let num_total_variable_parents = total_variable_parents(world);
        let mut next_parent_index = 0;
        let mut parent_index_starts = [0; 4 * Actions::NUM_ELEMENTS];

        let occurences = vec![0.0; num_total_variable_parents];

        let num_total_transitions = total_transitions(world);
        let mut next_transition_index = 0;
        let mut transition_starts = vec![0; num_total_variable_parents];
        let transitions = vec![0.0; num_total_transitions];

        for action_index in 0..Actions::NUM_ELEMENTS {
            let action = Actions::from_index(action_index).unwrap();

            {
                let num_x_parents = num_x_parents(world, action);
                let parent_index_start = &mut parent_index_starts[action_index * 4];

                *parent_index_start = next_parent_index;
                next_parent_index += num_x_parents;

                for t in 0..num_x_parents {
                    transition_starts[*parent_index_start + t] = next_transition_index;
                    next_transition_index += num_x_states;
                }
            }

            {
                let num_y_parents = num_y_parents(world, action);
                let parent_index_start = &mut parent_index_starts[action_index * 4 + 1];

                *parent_index_start = next_parent_index;
                next_parent_index += num_y_parents;

                for t in 0..num_y_parents {
                    transition_starts[*parent_index_start + t] = next_transition_index;
                    next_transition_index += num_y_states;
                }
            }

            {
                let num_passenger_parents = num_passenger_parents(world, action);
                let parent_index_start = &mut parent_index_starts[action_index * 4 + 2];

                *parent_index_start = next_parent_index;
                next_parent_index += num_passenger_parents;

                for t in 0..num_passenger_parents {
                    transition_starts[*parent_index_start + t] = next_transition_index;
                    next_transition_index += num_passenger_states;
                }
            }

            {
                let num_destination_parents = num_destination_parents(world, action);

                let parent_index_start = &mut parent_index_starts[action_index * 4 + 3];

                *parent_index_start = next_parent_index;
                next_parent_index += num_destination_parents;

                for t in 0..num_destination_parents {
                    transition_starts[*parent_index_start + t] = next_transition_index;
                    next_transition_index += num_destination_states;
                }
            }
        }

        assert_eq!(next_parent_index, num_total_variable_parents);
        assert_eq!(next_transition_index, num_total_transitions);

        Transitions {
            parent_index_starts,
            occurences,
            transition_starts,
            transitions,

            known_count,
        }
    }

    fn generate_x_parent_index(
        &self,
        world: &World,
        action: Actions,
        x_index: usize,
        y_index: usize,
    ) -> usize {
        let offset = match action {
            Actions::East | Actions::West => y_index * (world.width as usize) + x_index,
            _ => x_index,
        };

        self.parent_index_starts[action.to_index() * 4] + offset
    }

    fn generate_y_parent_index(&self, _world: &World, action: Actions, y_index: usize) -> usize {
        self.parent_index_starts[action.to_index() * 4 + 1] + y_index
    }

    fn generate_passenger_parent_index(
        &self,
        world: &World,
        action: Actions,
        x_index: usize,
        y_index: usize,
        passenger_index: usize,
        destination_index: usize,
    ) -> usize {
        let offset = match action {
            Actions::DropOff => {
                let mut result = y_index;

                result *= world.width as usize;
                result += x_index;

                let num_destination_states = world.num_fixed_positions();

                result *= num_destination_states;
                result += destination_index;

                result *= num_destination_states + 1;
                result += passenger_index;

                result
            }
            Actions::PickUp => {
                let mut result = y_index;

                result *= world.width as usize;
                result += x_index;

                let num_destination_states = world.num_fixed_positions();
                result *= num_destination_states + 1;
                result += passenger_index;

                result
            }
            _ => passenger_index,
        };

        self.parent_index_starts[action.to_index() * 4 + 2] + offset
    }

    fn generate_destination_parent_index(
        &self,
        _world: &World,
        action: Actions,
        destination_index: usize,
    ) -> usize {
        self.parent_index_starts[action.to_index() * 4 + 3] + destination_index
    }

    fn is_known(&self, parent_index: usize) -> bool {
        self.occurences[parent_index] >= self.known_count
    }

    fn get_transition(&self, parent_index: usize, next_index: usize) -> Option<f64> {
        let occurence_count = self.occurences[parent_index];

        if occurence_count >= self.known_count {
            let transition_count =
                self.transitions[self.transition_starts[parent_index] + next_index];

            Some(transition_count / occurence_count)
        } else {
            None
        }
    }

    fn apply_experience(&mut self, parent_index: usize, next_index: usize) {
        self.transitions[self.transition_starts[parent_index] + next_index] += 1.0;
        self.occurences[parent_index] += 1.0;
    }
}

#[derive(Debug, Clone)]
struct Rewards {
    reward_starts: [usize; Actions::NUM_ELEMENTS],
    occurences: Vec<f64>,
    rewards: Vec<f64>,

    known_count: f64,
}

impl Rewards {
    fn new(world: &World, known_count: f64) -> Rewards {
        let num_total_reward_parents = total_reward_parents(world);
        let mut next_reward_index = 0;
        let mut reward_starts = [0; Actions::NUM_ELEMENTS];

        for action_index in 0..Actions::NUM_ELEMENTS {
            let action = Actions::from_index(action_index).unwrap();

            reward_starts[action_index] = next_reward_index;
            next_reward_index += num_reward_parents(world, action);
        }

        assert_eq!(next_reward_index, num_total_reward_parents);

        let occurences = vec![0.0; num_total_reward_parents];
        let rewards = vec![0.0; num_total_reward_parents];

        Rewards {
            reward_starts,
            occurences,
            rewards,

            known_count,
        }
    }

    fn apply_experience(
        &mut self,
        reward: f64,
        world: &World,
        action: Actions,
        x_index: usize,
        y_index: usize,
        passenger_index: usize,
        destination_index: usize,
    ) {
        let reward_parent_index = generate_reward_parent_index(
            world,
            action,
            x_index,
            y_index,
            passenger_index,
            destination_index,
        );

        let action_index = action.to_index();

        let reward_index = self.reward_starts[action_index] + reward_parent_index;
        let count = &mut self.occurences[reward_index];
        if *count < self.known_count {
            *count += 1.0;

            let reward_entry = &mut self.rewards[reward_index];
            let delta = reward - *reward_entry;

            *reward_entry += delta / *count;
        }
    }

    fn get_reward(
        &self,
        world: &World,
        action: Actions,
        x_index: usize,
        y_index: usize,
        passenger_index: usize,
        destination_index: usize,
    ) -> Option<f64> {
        let reward_parent_index = generate_reward_parent_index(
            world,
            action,
            x_index,
            y_index,
            passenger_index,
            destination_index,
        );

        let action_index = action.to_index();

        let reward_index = self.reward_starts[action_index] + reward_parent_index;
        let count = self.occurences[reward_index];
        if count >= self.known_count {
            Some(self.rewards[reward_index])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct FactoredRMax {
    state_indexer: StateIndexer,
    rmax: f64,

    transitions: Transitions,
    rewards: Rewards,

    value_table: Vec<f64>,

    gamma: f64,
    error_delta: f64,
}

impl FactoredRMax {
    pub fn new(world: &World, gamma: f64, known_count: f64, error_delta: f64) -> FactoredRMax {
        let state_indexer = StateIndexer::new(world);
        let num_states = state_indexer.num_states();
        let value_table = vec![0.0; num_states];

        let transitions = Transitions::new(world, known_count);
        let rewards = Rewards::new(world, known_count);

        let rmax = if gamma < 1.0 {
            world.max_reward() / (1.0 - gamma)
        } else {
            world.max_reward()
        };

        FactoredRMax {
            state_indexer,
            rmax,

            transitions,
            rewards,

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
        next_state: &State,
        reward: f64,
    ) {
        let x_index = state.get_taxi().x as usize;
        let y_index = state.get_taxi().y as usize;

        let x_parent_index =
            self.transitions
                .generate_x_parent_index(world, action, x_index, y_index);

        let y_parent_index = self.transitions
            .generate_y_parent_index(world, action, y_index);

        if let Some(passenger_index) = generate_passenger_index(world, state) {
            if let Some(destination_index) = generate_destination_index(world, state) {
                let passenger_parent_index = self.transitions.generate_passenger_parent_index(
                    world,
                    action,
                    x_index,
                    y_index,
                    passenger_index,
                    destination_index,
                );

                let destination_parent_index = self.transitions.generate_destination_parent_index(
                    world,
                    action,
                    destination_index,
                );

                if !self.transitions.is_known(x_parent_index)
                    || !self.transitions.is_known(y_parent_index)
                    || !self.transitions.is_known(passenger_parent_index)
                    || !self.transitions.is_known(destination_parent_index)
                {
                    let next_x_index = next_state.get_taxi().x as usize;
                    self.transitions
                        .apply_experience(x_parent_index, next_x_index);

                    let next_y_index = next_state.get_taxi().y as usize;
                    self.transitions
                        .apply_experience(y_parent_index, next_y_index);

                    if let Some(next_passenger_index) = generate_passenger_index(world, next_state)
                    {
                        self.transitions
                            .apply_experience(passenger_parent_index, next_passenger_index);

                        if let Some(next_destination_index) =
                            generate_destination_index(world, next_state)
                        {
                            self.transitions
                                .apply_experience(destination_parent_index, next_destination_index);
                        }
                    }
                }

                self.rewards.apply_experience(
                    reward,
                    world,
                    action,
                    x_index,
                    y_index,
                    passenger_index,
                    destination_index,
                );
            }
        }
    }

    fn predict_transition(
        &self,
        world: &World,
        x_parent_index: usize,
        y_parent_index: usize,
        passenger_parent_index: usize,
        destination_parent_index: usize,
        next_state: &State,
    ) -> Option<f64> {
        let next_x_index = next_state.get_taxi().x as usize;
        let x_transition = self.transitions
            .get_transition(x_parent_index, next_x_index)?;

        let next_y_index = next_state.get_taxi().y as usize;
        let y_transition = self.transitions
            .get_transition(y_parent_index, next_y_index)?;

        let next_passenger_index = generate_passenger_index(world, next_state)?;
        let passenger_transition = self.transitions
            .get_transition(passenger_parent_index, next_passenger_index)?;

        let next_destination_index = generate_destination_index(world, next_state)?;
        let destination_transition = self.transitions
            .get_transition(destination_parent_index, next_destination_index)?;

        Some(x_transition * y_transition * destination_transition * passenger_transition)
    }

    fn measure_value(&self, world: &World, state: &State, action: Actions) -> f64 {
        if let Some(passenger_index) = generate_passenger_index(world, state) {
            if let Some(destination_index) = generate_destination_index(world, state) {
                let x_index = state.get_taxi().x as usize;
                let y_index = state.get_taxi().y as usize;

                let reward = match self.rewards.get_reward(
                    world,
                    action,
                    x_index,
                    y_index,
                    passenger_index,
                    destination_index,
                ) {
                    Some(reward) => reward,
                    None => self.rmax,
                };

                let x_parent_index =
                    self.transitions
                        .generate_x_parent_index(world, action, x_index, y_index);

                let y_parent_index =
                    self.transitions
                        .generate_y_parent_index(world, action, y_index);

                let passenger_parent_index = self.transitions.generate_passenger_parent_index(
                    world,
                    action,
                    x_index,
                    y_index,
                    passenger_index,
                    destination_index,
                );

                let destination_parent_index = self.transitions.generate_destination_parent_index(
                    world,
                    action,
                    destination_index,
                );

                let mut action_value = reward;

                for next_state_index in 0..self.state_indexer.num_states() {
                    let next_state = self.state_indexer
                        .get_state(world, next_state_index)
                        .unwrap();

                    if let Some(transition) = self.predict_transition(
                        world,
                        x_parent_index,
                        y_parent_index,
                        passenger_parent_index,
                        destination_parent_index,
                        &next_state,
                    ) {
                        action_value +=
                            transition * self.gamma * self.value_table[next_state_index];
                    } else if *state == next_state {
                        action_value += self.gamma * self.value_table[next_state_index];
                    }
                }
                action_value
            } else {
                panic!("Failed to find destination_index")
            }
        } else {
            panic!("Failed to find passenger_index");
        }
    }

    fn measure_best_value(&self, world: &World, state: &State) -> f64 {
        let mut best_value = -f64::MAX;

        for action_index in 0..Actions::NUM_ELEMENTS {
            let action = Actions::from_index(action_index).unwrap();

            let action_value = self.measure_value(world, state, action);

            if action_value > best_value {
                best_value = action_value;
            }
        }

        best_value
    }

    fn determine_best_action_index<R: Rng>(
        &self,
        world: &World,
        state: &State,
        rng: &mut R,
    ) -> usize {
        let mut best_value = -f64::MAX;
        let mut best_action_index = Actions::NUM_ELEMENTS;
        let mut num_found = 0;

        for action_index in 0..Actions::NUM_ELEMENTS {
            let action = Actions::from_index(action_index).unwrap();

            let action_value = self.measure_value(world, state, action);

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

    fn rebuild_value_table(&mut self, world: &World) {
        let num_states = self.state_indexer.num_states();

        for _ in 0..10_000 {
            let mut error = 0.0;

            for state_index in 0..num_states {
                let state = self.state_indexer.get_state(world, state_index).unwrap();

                let old_value = self.value_table[state_index];

                let new_value = self.measure_best_value(world, &state);

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

    fn select_best_action<R: Rng>(
        &self,
        world: &World,
        state: &State,
        rng: &mut R,
    ) -> Option<Actions> {
        let action_index = self.determine_best_action_index(world, state, rng);
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

            self.rebuild_value_table(world);

            if let Some(next_action) = self.select_best_action(world, &state, rng) {
                let (reward, next_state) = state.apply_action(world, next_action);

                self.apply_experience(world, &state, next_action, &next_state, reward);
                state = next_state;
            } else {
                return None;
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

            if let Some(next_action) = self.select_best_action(world, &state, rng) {
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

            if let Some(next_action) = self.select_best_action(world, &state, rng) {
                let (_, next_state) = state.apply_action(world, next_action);
                state = next_state;
            } else {
                break;
            }
        }

        state.at_destination()
    }

    fn report_training_result(&self, _world: &World) {
        // let mut rng = Isaac64Rng::new_unseeded();

        // let num_states = self.state_indexer.num_states();
        // for state_index in 0..num_states {
        //     let state = self.state_indexer.get_state(world, state_index).unwrap();

        //     if !state.at_destination() {
        //         if let Some(next_action) = self.select_best_action(world, &state, &mut rng) {
        //             println!("===================");
        //             println!("{}", state.display(world));
        //             println!("Best action: {}", next_action);

        //             for action_index in 0..Actions::NUM_ELEMENTS {
        //                 let action = Actions::from_index(action_index).unwrap();

        //                 let reward = self.get_reward(world, &state, action);
        //                 let mut action_value = 0.0;

        //                 for next_state_index in 0..self.state_indexer.num_states() {
        //                     let next_state = self.state_indexer
        //                         .get_state(world, next_state_index)
        //                         .unwrap();

        //                     let (transition, reward) =
        //                         self.predict_transition_reward(world, &state, action, &next_state);

        //                     action_value += transition
        //                         * (reward + self.gamma * self.value_table[next_state_index]);
        //                 }

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

fn num_x_parents(world: &World, action: Actions) -> usize {
    match action {
        Actions::East | Actions::West => (world.width * world.height) as usize,
        _ => world.width as usize,
    }
}

fn total_x_parents(world: &World) -> usize {
    2 * (world.width * world.height) as usize + (Actions::NUM_ELEMENTS - 2) * world.width as usize
}

fn num_y_parents(world: &World, _: Actions) -> usize {
    world.height as usize
}

fn total_y_parents(world: &World) -> usize {
    Actions::NUM_ELEMENTS * world.height as usize
}

fn generate_passenger_index(world: &World, state: &State) -> Option<usize> {
    match state.get_passenger() {
        None => Some(0),
        Some(passenger_id) => world.get_fixed_index(passenger_id).map(|i| i + 1),
    }
}

fn num_passenger_parents(world: &World, action: Actions) -> usize {
    let num_destination_states = world.num_fixed_positions() as usize;
    let num_passenger_states = (num_destination_states + 1) as usize;

    let num_taxi_states = (world.height * world.width) as usize;

    match action {
        Actions::DropOff => num_taxi_states * num_destination_states * num_passenger_states,
        Actions::PickUp => num_taxi_states * num_passenger_states,
        _ => num_passenger_states,
    }
}

fn total_passenger_parents(world: &World) -> usize {
    let num_destination_states = world.num_fixed_positions() as usize;
    let num_passenger_states = (num_destination_states + 1) as usize;

    let num_taxi_states = (world.height * world.width) as usize;

    num_taxi_states * num_destination_states * num_passenger_states
        + num_taxi_states * num_passenger_states
        + (Actions::NUM_ELEMENTS - 2) * num_passenger_states
}

fn num_destination_parents(world: &World, _: Actions) -> usize {
    world.num_fixed_positions()
}

fn total_destination_parents(world: &World) -> usize {
    Actions::NUM_ELEMENTS * world.num_fixed_positions()
}

fn generate_destination_index(world: &World, state: &State) -> Option<usize> {
    world.get_fixed_index(state.get_destination())
}

fn total_variable_parents(world: &World) -> usize {
    total_x_parents(world) + total_y_parents(world) + total_passenger_parents(world)
        + total_destination_parents(world)
}

fn total_transitions(world: &World) -> usize {
    let num_x_states = world.width as usize;
    let num_y_states = world.height as usize;
    let num_destination_states = world.num_fixed_positions();
    let num_passenger_states = num_destination_states + 1;

    total_x_parents(world) * num_x_states + total_y_parents(world) * num_y_states
        + total_passenger_parents(world) * num_passenger_states
        + total_destination_parents(world) * num_destination_states
}

fn num_reward_parents(world: &World, action: Actions) -> usize {
    let num_taxi_values = (world.width * world.height) as usize;
    let num_destination_values = world.num_fixed_positions();
    let num_passenger_values = num_destination_values + 1;
    match action {
        Actions::North | Actions::South | Actions::East | Actions::West => num_taxi_values,

        Actions::PickUp => num_passenger_values * num_taxi_values,

        Actions::DropOff => num_destination_values * num_passenger_values * num_taxi_values,
    }
}

fn total_reward_parents(world: &World) -> usize {
    let num_taxi_values = (world.width * world.height) as usize;
    let num_destination_values = world.num_fixed_positions();
    let num_passenger_values = num_destination_values + 1;

    num_taxi_values * 4 + num_passenger_values * num_taxi_values
        + num_destination_values * num_passenger_values * num_taxi_values
}

fn generate_reward_parent_index(
    world: &World,
    action: Actions,
    x_index: usize,
    y_index: usize,
    passenger_index: usize,
    destination_index: usize,
) -> usize {
    match action {
        Actions::North | Actions::South | Actions::East | Actions::West => {
            y_index * (world.width as usize) + x_index
        }

        Actions::PickUp => {
            let mut result = passenger_index;

            result *= world.height as usize;
            result += y_index;

            result *= world.width as usize;
            result += x_index;

            result
        }

        Actions::DropOff => {
            let mut result = destination_index;

            result *= world.num_fixed_positions() + 1;
            result += passenger_index;

            result *= world.height as usize;
            result += y_index;

            result *= world.width as usize;
            result += x_index;

            result
        }
    }
}

#[cfg(test)]
mod test_factoredrmax {

    use rand::Isaac64Rng;
    use super::*;

    #[test]
    fn learn_simple() {
        let world_str = "\
                         ┌───┐\n\
                         │R .│\n\
                         │   │\n\
                         │. G│\n\
                         └───┘\n\
                         ";

        let world = World::build_from_str(world_str).unwrap();

        let mut factoredrmax = FactoredRMax::new(&world, 0.3, 1.0, 1.0e-6);

        let state = State::build(&world, (0, 1), Some('R'), 'G').unwrap();

        let expected_initial_state = "\
                                      ┌───┐\n\
                                      │p .│\n\
                                      │   │\n\
                                      │t d│\n\
                                      └───┘\n\
                                      ";

        let initial_state = state.clone();
        assert_eq!(expected_initial_state, initial_state.display(&world));

        let mut rng = Isaac64Rng::new_unseeded();

        let result = factoredrmax.learn(&world, state, 100, &mut rng);
        assert!(result.is_some());
    }
}
