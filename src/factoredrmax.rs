
use std::f64;
use std::cmp;

use rand::Rng;
use float_cmp::ApproxOrdUlps;

use state::State;
use actions::Actions;
use world::World;

use runner::{Runner, Attempt};
use state_indexer::StateIndexer;

#[derive(Debug, Clone, Copy, Default)]
struct RewardEntry {
    mean: f64,
    count: f64,
}


#[derive(Debug, Clone)]
pub struct FactoredRMax {
    state_indexer: StateIndexer,
    rmax: f64,

    transition_occurences: Vec<Vec<Vec<f64>>>, // [Action][variable][parent_values]
    transitions: Vec<Vec<Vec<Vec<f64>>>>, // [Action][variable][parent_values][destination_value]

    reward_table: Vec<Vec<RewardEntry>>, // [Action][parent_values]

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

        let num_x_states = world.width as usize;
        let num_y_states = world.height as usize;
        let num_destination_states = world.num_fixed_positions();
        let num_passenger_states = num_destination_states + 1;

        let mut transitions = Vec::with_capacity(Actions::NUM_ELEMENTS);
        let mut transition_occurences = Vec::with_capacity(Actions::NUM_ELEMENTS);
        let mut reward_table = Vec::with_capacity(Actions::NUM_ELEMENTS);

        for action_index in 0..Actions::NUM_ELEMENTS {

            let action = Actions::from_index(action_index).unwrap();

            let occurence_entry = vec![
                vec![0.0; num_x_parents(world, action)],
                vec![0.0; num_y_parents(world, action)],
                vec![0.0; num_passenger_parents(world, action)],
                vec![0.0; num_destination_parents(world, action)],
            ];

            transition_occurences.push(occurence_entry);

            let entry =
                vec![
                    vec![vec![0.0; num_x_states]; num_x_parents(world, action)],
                    vec![vec![0.0; num_y_states]; num_y_parents(world, action)],
                    vec![vec![0.0; num_passenger_states]; num_passenger_parents(world, action)],
                    vec![vec![0.0; num_destination_states]; num_destination_parents(world, action)],
                ];

            transitions.push(entry);

            let reward_entry = vec![RewardEntry::default(); num_reward_parents(world, action)];
            reward_table.push(reward_entry);

        }

        FactoredRMax {
            state_indexer,
            rmax: world.max_reward() / (1.0 - gamma),

            transition_occurences,
            transitions,

            reward_table,

            value_table,

            gamma,
            known_count,
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

        let action_index = action.to_index();

        let occurences = &mut self.transition_occurences[action_index];

        let x_index = state.get_taxi().x as usize;
        let y_index = state.get_taxi().y as usize;

        let x_parent_index = generate_x_parent_index(world, x_index, y_index, action);

        let x_occurence_count = occurences[0][x_parent_index];
        let y_occurence_count = occurences[1][y_index];

        if let Some(passenger_index) = generate_passenger_index(world, state) {

            if let Some(destination_index) = generate_destination_index(world, state) {

                let passenger_parent_index = generate_passenger_parent_index(
                    world,
                    x_index,
                    y_index,
                    passenger_index,
                    destination_index,
                    action,
                );
                let passenger_occurence_count = occurences[2][passenger_parent_index];

                let destination_occurence_count = occurences[3][destination_index];

                if x_occurence_count < self.known_count || y_occurence_count < self.known_count ||
                    passenger_occurence_count < self.known_count ||
                    destination_occurence_count < self.known_count
                {

                    let transitions = &mut self.transitions[action_index];

                    let next_x_index = next_state.get_taxi().x as usize;
                    transitions[0][generate_x_parent_index(world, x_index, y_index, action)]
                        [next_x_index] += 1.0;
                    occurences[0][x_parent_index] += 1.0;

                    let next_y_index = next_state.get_taxi().y as usize;
                    transitions[1][y_index][next_y_index] += 1.0;
                    occurences[1][y_index] += 1.0;

                    if let Some(next_passenger_index) =
                        generate_passenger_index(world, next_state)
                    {

                        transitions[2][generate_passenger_parent_index(
                            world,
                            x_index,
                            y_index,
                            passenger_index,
                            destination_index,
                            action,
                        )]
                            [next_passenger_index] += 1.0;

                        occurences[2][passenger_parent_index] += 1.0;

                        if let Some(next_destination_index) =
                            generate_destination_index(world, next_state)
                        {

                            transitions[3][destination_index][next_destination_index] += 1.0;
                            occurences[3][destination_index] += 1.0;
                        }
                    }
                }

                let reward_parent_index = generate_reward_parent_index(
                    world,
                    x_index,
                    y_index,
                    passenger_index,
                    destination_index,
                    action,
                );

                let reward_entry = &mut self.reward_table[action_index][reward_parent_index];
                if reward_entry.count < self.known_count {
                    reward_entry.count += 1.0;

                    let delta = reward - reward_entry.mean;

                    reward_entry.mean += delta / reward_entry.count;
                }
            }
        }
    }

    fn predict_transition(
        &self,
        world: &World,
        x_index: usize,
        y_index: usize,
        passenger_index: usize,
        destination_index: usize,
        action: Actions,
        next_state: &State,
    ) -> Option<f64> {

        let action_index = action.to_index();

        let occurences = &self.transition_occurences[action_index];

        let x_occurence_count =
            occurences[0][generate_x_parent_index(world, x_index, y_index, action)];

        if x_occurence_count < self.known_count {
            return None;
        }

        let y_occurence_count = occurences[1][y_index];
        if y_occurence_count < self.known_count {
            return None;
        }

        let passenger_occurence_count = occurences[2][generate_passenger_parent_index(
            world,
            x_index,
            y_index,
            passenger_index,
            destination_index,
            action,
        )];
        if passenger_occurence_count < self.known_count {
            return None;
        }

        let destination_occurence_count = occurences[3][destination_index];

        if destination_occurence_count < self.known_count {
            return None;
        }

        let transitions = &self.transitions[action_index];

        let next_x_index = next_state.get_taxi().x as usize;
        let x_transition =
            transitions[0][generate_x_parent_index(world, x_index, y_index, action)]
                [next_x_index] / x_occurence_count;

        let next_y_index = next_state.get_taxi().y as usize;
        let y_transition = transitions[1][y_index][next_y_index] / y_occurence_count;

        if let Some(next_passenger_index) = generate_passenger_index(world, next_state) {

            let passenger_transition = transitions[2][generate_passenger_parent_index(
                world,
                x_index,
                y_index,
                passenger_index,
                destination_index,
                action,
            )]
                [next_passenger_index] /
                passenger_occurence_count;

            if let Some(next_destination_index) = generate_destination_index(world, next_state) {

                let destination_transition = transitions[3][destination_index]
                    [next_destination_index] /
                    destination_occurence_count;

                return Some(
                    x_transition * y_transition * destination_transition * passenger_transition,
                );
            }
        }

        None
    }

    fn get_reward(&self, world: &World, state: &State, action: Actions) -> f64 {
        let action_index = action.to_index();

        if let Some(passenger_index) = generate_passenger_index(world, state) {

            if let Some(destination_index) = generate_destination_index(world, state) {

                let x_index = state.get_taxi().x as usize;
                let y_index = state.get_taxi().y as usize;

                let reward_parent_index = generate_reward_parent_index(
                    world,
                    x_index,
                    y_index,
                    passenger_index,
                    destination_index,
                    action,
                );

                let reward_entry = self.reward_table[action_index][reward_parent_index];

                if reward_entry.count >= self.known_count {
                    return reward_entry.mean;
                }
            }
        }

        self.rmax
    }

    fn predict_transition_reward(
        &self,
        world: &World,
        state: &State,
        action: Actions,
        next_state: &State,
    ) -> (f64, f64) {

        let action_index = action.to_index();

        if let Some(passenger_index) = generate_passenger_index(world, state) {

            if let Some(destination_index) = generate_destination_index(world, state) {

                let x_index = state.get_taxi().x as usize;
                let y_index = state.get_taxi().y as usize;

                let reward_parent_index = generate_reward_parent_index(
                    world,
                    x_index,
                    y_index,
                    passenger_index,
                    destination_index,
                    action,
                );

                let reward_entry = &self.reward_table[action_index][reward_parent_index];

                if reward_entry.count >= self.known_count {

                    let reward = reward_entry.mean;

                    if let Some(transition_value) =
                        self.predict_transition(
                            world,
                            x_index,
                            y_index,
                            passenger_index,
                            destination_index,
                            action,
                            next_state,
                        )
                    {
                        return (transition_value, reward);
                    }
                }
            }
        }



        let transition = if state == next_state { 1.0 } else { 0.0 };

        (transition, self.rmax)
    }

    fn measure_best_value(&self, world: &World, state: &State) -> f64 {

        let mut best_value = -f64::MAX;

        for action_index in 0..Actions::NUM_ELEMENTS {

            let action = Actions::from_index(action_index).unwrap();


            let mut action_value = self.get_reward(world, state, action);

            for next_state_index in 0..self.state_indexer.num_states() {

                let next_state = self.state_indexer
                    .get_state(world, next_state_index)
                    .unwrap();

                let (transition, reward) =
                    self.predict_transition_reward(world, state, action, &next_state);

                action_value += transition *
                    (reward + self.gamma * self.value_table[next_state_index]);
            }

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

            let mut action_value = self.get_reward(world, state, action);

            for next_state_index in 0..self.state_indexer.num_states() {

                let next_state = self.state_indexer
                    .get_state(world, next_state_index)
                    .unwrap();


                let (transition, reward) =
                    self.predict_transition_reward(world, state, action, &next_state);
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

    fn report_training_result(&self, _world: &World) {}
}

fn num_x_parents(world: &World, action: Actions) -> usize {
    match action {
        Actions::East | Actions::West => (world.width * world.height) as usize,
        _ => world.width as usize,
    }
}


fn generate_x_parent_index(
    world: &World,
    x_index: usize,
    y_index: usize,
    action: Actions,
) -> usize {
    match action {
        Actions::East | Actions::West => y_index * (world.width as usize) + x_index,
        _ => x_index,
    }
}

fn num_y_parents(world: &World, _: Actions) -> usize {
    world.height as usize
}

fn generate_passenger_index(world: &World, state: &State) -> Option<usize> {
    match state.get_passenger() {
        Some(passenger_id) => world.get_fixed_index(passenger_id),
        None => Some(world.num_fixed_positions()),
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

fn generate_passenger_parent_index(
    world: &World,
    x_index: usize,
    y_index: usize,
    passenger_index: usize,
    destination_index: usize,
    action: Actions,
) -> usize {


    match action {
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
    }
}

fn num_destination_parents(world: &World, _: Actions) -> usize {
    world.num_fixed_positions()
}

fn generate_destination_index(world: &World, state: &State) -> Option<usize> {
    world.get_fixed_index(state.get_destination())
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

fn generate_reward_parent_index(
    world: &World,
    x_index: usize,
    y_index: usize,
    passenger_index: usize,
    destination_index: usize,
    action: Actions,
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
