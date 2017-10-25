
use rand::{Rng, thread_rng};


use state::State;
use actions::Actions;
use world::{World, ActionAffect};

use runner::{Runner, Attempt};

struct StateIndexer {
    num_taxi_states: usize,
    num_passenger_states: usize,
    num_destination_states: usize,
}

impl StateIndexer {
    fn new(world: &World) -> StateIndexer {
        let num_taxi_states = (world.width * world.height) as usize;
        let num_passenger_states = world.num_fixed_positions() + 1;
        let num_destination_states = world.num_fixed_positions();

        StateIndexer {
            num_taxi_states,
            num_passenger_states,
            num_destination_states,
        }
    }

    fn num_states(&self) -> usize {
        self.num_taxi_states * self.num_passenger_states * self.num_destination_states
    }

    fn get_index(&self, world: &World, state: &State) -> Option<usize> {

        if let Some(destination_index) = world.get_fixed_index(state.get_destination()) {

            let mut result = destination_index;

            if let Some(passenger_index) =
                match state.get_passenger() {
                    Some(passenger_id) => world.get_fixed_index(passenger_id),
                    None => Some(self.num_passenger_states - 1),
                }
            {
                result *= self.num_passenger_states;
                result += passenger_index;

                let taxi_pos = state.get_taxi();
                let taxi_index = (world.width * taxi_pos.y + taxi_pos.x) as usize;

                result *= self.num_taxi_states;
                result += taxi_index;

                return Some(result);
            }
        }

        None
    }

    fn get_state(&self, world: &World, mut state_index: usize) -> Option<State> {

        let taxi_index = state_index % self.num_taxi_states;
        state_index /= self.num_taxi_states;

        let passenger_index = state_index % self.num_passenger_states;
        state_index /= self.num_passenger_states;

        let destination_index = state_index;

        let taxi_x = taxi_index % (world.width as usize);
        let taxi_y = taxi_index / (world.width as usize);

        if let Some(destination) = world.get_fixed_id_from_index(destination_index) {

            let passenger = if passenger_index < world.num_fixed_positions() {
                world.get_fixed_id_from_index(passenger_index)
            } else {
                None
            };

            State::build(
                &world,
                (taxi_x as i32, taxi_y as i32),
                passenger,
                destination,
            ).ok()

        } else {
            None
        }
    }
}

pub struct QLearner {
    alpha: f64,
    gamma: f64,
    epsilon: f64,

    movement_cost: f64,
    miss_passenger_cost: f64,

    state_indexer: StateIndexer,
    qtable: Vec<[f64; Actions::NUM_ELEMENTS]>,
    show_table: bool,
}

impl QLearner {
    pub fn new(world: &World) -> QLearner {

        let state_indexer = StateIndexer::new(&world);
        let num_states = state_indexer.num_states();
        let mut qtable = Vec::with_capacity(num_states);

        for _ in 0..num_states {
            qtable.push([0.0f64; Actions::NUM_ELEMENTS]);
        }

        QLearner {
            alpha: 1.0,
            gamma: 1.0,
            epsilon: 1.0,

            movement_cost: -1.0,
            miss_passenger_cost: -10.0,

            state_indexer,
            qtable,
            show_table: false,
        }
    }

    fn determine_reward(&self, world: &World, state: &State, next_action: Actions) -> f64 {
        match world.determine_affect(state.get_taxi(), next_action) {
            ActionAffect::Move(_) => self.movement_cost,
            ActionAffect::Invalid => {
                match next_action {
                    Actions::North | Actions::South | Actions::East | Actions::West => {
                        self.movement_cost
                    }
                    Actions::PickUp | Actions::DropOff => self.miss_passenger_cost,
                }
            }
            ActionAffect::PickUp(id) => {
                match state.get_passenger() {
                    Some(passenger_id) if passenger_id == id => 0.0,
                    _ => self.miss_passenger_cost,
                }
            }
            ActionAffect::DropOff(id) => {
                if state.get_passenger() == None && id == state.get_destination() {
                    0.0
                } else {
                    self.miss_passenger_cost
                }
            }
        }
    }

    fn determine_greedy_action<R: Rng>(&self, state_index: usize, rng: &mut R) -> Actions {
        let mut num_found = 0;
        let mut best_action = None;
        let mut best_value = 0.0;

        let values = &self.qtable[state_index];

        for (i, value) in values.iter().enumerate() {

            if best_action == None {
                best_action = Actions::from_index(i);
                best_value = *value;
                num_found = 1;
            } else {
                let value_diff = value - best_value;

                if value_diff > 1e-10f64 {
                    best_action = Actions::from_index(i);
                    best_value = *value;
                    num_found = 1;
                } else if value_diff > -1e-10f64 {

                    num_found += 1;

                    if rng.gen_range(0, num_found) == 0 {
                        best_action = Actions::from_index(i);
                    }
                }
            }
        }

        best_action.unwrap()
    }

    fn determine_learning_action<R: Rng>(&self, state_index: usize, mut rng: &mut R) -> Actions {

        let nongreedy_roll = rng.gen_range(0.0f64, 1.0f64);

        if nongreedy_roll < self.epsilon {
            Actions::from_index(rng.gen_range(0, Actions::NUM_ELEMENTS)).unwrap()
        } else {
            self.determine_greedy_action(state_index, &mut rng)
        }
    }

    fn find_maximal_value(&self, state_index: usize) -> Option<f64> {

        let state_values = &self.qtable[state_index];

        let mut best_value: Option<f64> = None;

        for value in state_values {

            best_value = Some(if let Some(current_best) = best_value {
                if current_best < *value {
                    *value
                } else {
                    current_best
                }
            } else {
                *value
            });
        }

        best_value
    }

    fn apply_experience(
        &mut self,
        state_index: usize,
        next_action: Actions,
        next_state_index: usize,
        reward: f64,
    ) {
        if let Some(next_state_value) = self.find_maximal_value(next_state_index) {

            let state_values = &mut self.qtable[state_index];
            let action_entry = &mut state_values[next_action.to_index()];

            if self.alpha > 0.0 {
                *action_entry *= 1.0 - self.alpha;
            }

            *action_entry += self.alpha * (reward + self.gamma * next_state_value);
        }

    }
}

impl Runner for QLearner {
    fn learn(&mut self, world: &World, mut state: State, max_steps: usize) -> Option<usize> {

        let mut rng = thread_rng();

        for step in 0..max_steps {
            if state.at_destination() {
                return Some(step);
            }

            if let Some(state_index) = self.state_indexer.get_index(&world, &state) {

                let next_action = self.determine_learning_action(state_index, &mut rng);
                let reward = self.determine_reward(&world, &state, next_action);

                state = state.apply_action(&world, next_action);

                if let Some(next_state_index) = self.state_indexer.get_index(&world, &state) {
                    self.apply_experience(state_index, next_action, next_state_index, reward);
                }

            } else {
                return None;
            }
        }

        None

    }

    fn attempt(&self, world: &World, mut state: State, max_steps: usize) -> Attempt {
        let mut rng = thread_rng();
        let mut attempt = Attempt::new(state.clone(), max_steps);

        for _ in 0..max_steps {
            if state.at_destination() {
                break;
            }

            if let Some(state_index) = self.state_indexer.get_index(&world, &state) {

                let next_action = self.determine_greedy_action(state_index, &mut rng);
                attempt.step(next_action);

                state = state.apply_action(&world, next_action);

            } else {
                break;
            }
        }

        if state.at_destination() {
            attempt.succeeded()
        }

        attempt
    }

    fn report_training_result(&self, world: &World) {

        if self.show_table {
            println!("");
            for (i, action_values) in self.qtable.iter().enumerate() {
                let state = self.state_indexer.get_state(&world, i).unwrap();
                println!("{}", state.display(&world));
                println!("{:?}", action_values);
            }
        }
    }
}

#[cfg(test)]
mod test_qlearner {

    use super::*;

    #[test]
    fn indices_unique() {
        let source_world = "\
        ┌─────┐\n\
        │R . G│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. Y .│\n\
        └─────┘\n\
        ";

        let possible_passengers = [Some('R'), Some('G'), Some('Y'), None];

        let possible_destinations = ['R', 'Y', 'G'];


        let world = World::build_from_str(&source_world).unwrap();

        let state_indexer = StateIndexer::new(&world);

        let mut visited_states = vec![false; state_indexer.num_states()];

        for destination in &possible_destinations {
            for passenger in &possible_passengers {
                for y in 0..world.height {
                    for x in 0..world.width {

                        let state = State::build(&world, (x, y), *passenger, *destination).unwrap();
                        let state_index = state_indexer.get_index(&world, &state).unwrap();

                        assert!(state_index < visited_states.len());
                        assert!(!visited_states[state_index]);

                        let reconstructed_state =
                            state_indexer.get_state(&world, state_index).unwrap();

                        println!(
                            "---- {} ----\n{}\n{}",
                            state_index,
                            state.display(&world),
                            reconstructed_state.display(&world),
                        );

                        assert_eq!(state.display(&world), reconstructed_state.display(&world));

                        visited_states[state_index] = true;
                    }
                }
            }
        }

        for (i, v) in visited_states.iter().enumerate() {
            if !v {
                println!("State index {} not visited.", i);
            }

            assert!(v);
        }
    }

    #[test]
    fn movement_reward() {
        let source_world = "\
        ┌─────┐\n\
        │R . G│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. Y .│\n\
        └─────┘\n\
        ";

        let expected_initial_str = "\
        ┌─────┐\n\
        │p . d│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. t .│\n\
        └─────┘\n\
        ";

        let world = World::build_from_str(source_world).unwrap();
        let initial_state = State::build(&world, (1, 2), Some('R'), 'G').unwrap();

        assert_eq!(expected_initial_str, initial_state.display(&world));

        let qlearner = QLearner::new(&world);

        let north_reward = qlearner.determine_reward(&world, &initial_state, Actions::North);
        assert_eq!(-1.0, north_reward);

        let south_reward = qlearner.determine_reward(&world, &initial_state, Actions::South);
        assert_eq!(-1.0, south_reward);

        let east_reward = qlearner.determine_reward(&world, &initial_state, Actions::East);
        assert_eq!(-1.0, east_reward);

        let west_reward = qlearner.determine_reward(&world, &initial_state, Actions::West);
        assert_eq!(-1.0, west_reward);
    }

    #[test]
    fn correct_pickup_reward() {
        let source_world = "\
        ┌─────┐\n\
        │R . G│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. Y .│\n\
        └─────┘\n\
        ";

        let expected_initial_str = "\
        ┌─────┐\n\
        │p . d│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. . .│\n\
        └─────┘\n\
        ";

        let world = World::build_from_str(source_world).unwrap();
        let initial_state = State::build(&world, (0, 0), Some('R'), 'G').unwrap();

        assert_eq!(expected_initial_str, initial_state.display(&world));

        let qlearner = QLearner::new(&world);

        let pickup_reward = qlearner.determine_reward(&world, &initial_state, Actions::PickUp);
        assert_eq!(0.0, pickup_reward);
    }

    #[test]
    fn incorrect_pickup_reward() {
        let source_world = "\
        ┌─────┐\n\
        │R . G│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. Y .│\n\
        └─────┘\n\
        ";

        let world = World::build_from_str(source_world).unwrap();
        let qlearner = QLearner::new(&world);

        let expected_off_passenger_str = "\
        ┌─────┐\n\
        │p . d│\n\
        │     │\n\
        │. t .│\n\
        │     │\n\
        │. . .│\n\
        └─────┘\n\
        ";

        let off_passenger_state = State::build(&world, (1, 1), Some('R'), 'G').unwrap();

        assert_eq!(
            expected_off_passenger_str,
            off_passenger_state.display(&world)
        );
        assert_eq!(
            -10.0,
            qlearner.determine_reward(&world, &off_passenger_state, Actions::PickUp)
        );

        let expected_has_passenger_str = "\
        ┌─────┐\n\
        │. . d│\n\
        │     │\n\
        │. T .│\n\
        │     │\n\
        │. . .│\n\
        └─────┘\n\
        ";

        let has_passenger_state = State::build(&world, (1, 1), None, 'G').unwrap();

        assert_eq!(
            expected_has_passenger_str,
            has_passenger_state.display(&world)
        );
        assert_eq!(
            -10.0,
            qlearner.determine_reward(&world, &has_passenger_state, Actions::PickUp)
        );

        let expected_wrong_fp_str = "\
        ┌─────┐\n\
        │p . d│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. t .│\n\
        └─────┘\n\
        ";

        let wrong_fp_state = State::build(&world, (1, 2), Some('R'), 'G').unwrap();

        assert_eq!(expected_wrong_fp_str, wrong_fp_state.display(&world));
        assert_eq!(
            -10.0,
            qlearner.determine_reward(&world, &wrong_fp_state, Actions::PickUp)
        );
    }

    #[test]
    fn incorrect_dropoff_reward() {
        let source_world = "\
        ┌─────┐\n\
        │R . G│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. Y .│\n\
        └─────┘\n\
        ";

        let world = World::build_from_str(source_world).unwrap();
        let qlearner = QLearner::new(&world);

        let expected_no_passenger_str = "\
        ┌─────┐\n\
        │p . d│\n\
        │     │\n\
        │. t .│\n\
        │     │\n\
        │. . .│\n\
        └─────┘\n\
        ";

        let no_passenger_state = State::build(&world, (1, 1), Some('R'), 'G').unwrap();

        assert_eq!(
            expected_no_passenger_str,
            no_passenger_state.display(&world)
        );

        assert_eq!(
            -10.0,
            qlearner.determine_reward(&world, &no_passenger_state, Actions::DropOff)
        );

        let expected_no_passenger_on_dest_str = "\
        ┌─────┐\n\
        │p . d│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. . .│\n\
        └─────┘\n\
        ";

        let no_passenger_on_deststate = State::build(&world, (2, 0), Some('R'), 'G').unwrap();

        assert_eq!(
            expected_no_passenger_on_dest_str,
            no_passenger_on_deststate.display(&world)
        );

        assert_eq!(
            -10.0,
            qlearner.determine_reward(&world, &no_passenger_on_deststate, Actions::DropOff)
        );

        let expected_passenger_off_fp_str = "\
        ┌─────┐\n\
        │. . d│\n\
        │     │\n\
        │. T .│\n\
        │     │\n\
        │. . .│\n\
        └─────┘\n\
        ";

        let passenger_off_fp_state = State::build(&world, (1, 1), None, 'G').unwrap();

        assert_eq!(
            expected_passenger_off_fp_str,
            passenger_off_fp_state.display(&world)
        );

        assert_eq!(
            -10.0,
            qlearner.determine_reward(&world, &passenger_off_fp_state, Actions::DropOff)
        );

        let expected_passenger_wrong_fp_str = "\
        ┌─────┐\n\
        │. . d│\n\
        │     │\n\
        │. . .│\n\
        │     │\n\
        │. T .│\n\
        └─────┘\n\
        ";

        let passenger_wrong_fp_state = State::build(&world, (1, 2), None, 'G').unwrap();

        assert_eq!(
            expected_passenger_wrong_fp_str,
            passenger_wrong_fp_state.display(&world)
        );

        assert_eq!(
            -10.0,
            qlearner.determine_reward(&world, &passenger_wrong_fp_state, Actions::DropOff)
        );
    }

    #[test]
    fn learns_go_north() {
        let world_str = "\
		┌───┐\n\
		│R .│\n\
		│   │\n\
		│. G│\n\
		└───┘\n\
		";

        let world = World::build_from_str(world_str).unwrap();

        let expected_initial_str = "\
		┌───┐\n\
		│p .│\n\
		│   │\n\
		│t d│\n\
		└───┘\n\
		";

        let mut rng = thread_rng();

        let initial_state = State::build(&world, (0, 1), Some('R'), 'G').unwrap();
        assert_eq!(expected_initial_str, initial_state.display(&world));

        let mut qlearner = QLearner::new(&world);

        let south_state = initial_state.apply_action(&world, Actions::South);
        assert_eq!(expected_initial_str, south_state.display(&world));

        let initial_index = qlearner
            .state_indexer
            .get_index(&world, &initial_state)
            .unwrap();
        let reward = qlearner.determine_reward(&world, &initial_state, Actions::South);
        let south_index = qlearner
            .state_indexer
            .get_index(&world, &south_state)
            .unwrap();

        assert_eq!(south_index, initial_index);

        qlearner.apply_experience(initial_index, Actions::South, south_index, reward);

        let best_action = qlearner.determine_greedy_action(south_index, &mut rng);
        assert!(best_action != Actions::South);
        println!("Chose action {:?}", best_action);

        println!("");
        for row in qlearner.qtable {
            println!("{:?}", row);
        }
    }

    #[test]
    fn initial_greedy_action_is_random() {

        let world_str = "\
		┌───┐\n\
		│R .│\n\
		│   │\n\
		│. G│\n\
		└───┘\n\
		";

        let world = World::build_from_str(world_str).unwrap();

        let qlearner = QLearner::new(&world);

        let mut counts = vec![0.0f64; Actions::NUM_ELEMENTS];

        let mut rng = thread_rng();
        let max_iterations = 100000;

        for _ in 0..max_iterations {

            let action: Actions = qlearner.determine_greedy_action(0, &mut rng);

            counts[action.to_index()] += 1.0;
        }

        // chi-squared should not exceed this for 95% confidence.
        let p_05 = 11.07;

        let expected_count = (max_iterations as f64) / (counts.len() as f64);

        let mut chi_sqr = 0.0f64;
        for count in &counts {
            let delta = count - expected_count;
            chi_sqr += (delta * delta) / expected_count;
        }

        println!("");
        println!(
            "north count = {}, ratio = {}",
            counts[Actions::North.to_index()],
            counts[Actions::North.to_index()] / expected_count
        );

        println!(
            "south count = {}, ratio = {}",
            counts[Actions::South.to_index()],
            counts[Actions::South.to_index()] / expected_count
        );

        println!(
            "east count = {}, ratio = {}",
            counts[Actions::East.to_index()],
            counts[Actions::East.to_index()] / expected_count
        );

        println!(
            "west count = {}, ratio = {}",
            counts[Actions::West.to_index()],
            counts[Actions::West.to_index()] / expected_count
        );

        println!(
            "pickup count = {}, ratio = {}",
            counts[Actions::PickUp.to_index()],
            counts[Actions::PickUp.to_index()] / expected_count
        );


        println!(
            "dropoff count = {}, ratio = {}",
            counts[Actions::DropOff.to_index()],
            counts[Actions::DropOff.to_index()] / expected_count
        );

        println!("chi-squared = {}, 95% confidence = {}", chi_sqr, p_05);

        assert!(chi_sqr < p_05);
    }

    #[test]
    fn initial_learning_action_is_random() {

        let world_str = "\
		┌───┐\n\
		│R .│\n\
		│   │\n\
		│. G│\n\
		└───┘\n\
		";

        let world = World::build_from_str(world_str).unwrap();

        let qlearner = QLearner::new(&world);

        let mut counts = vec![0.0f64; Actions::NUM_ELEMENTS];

        assert!(counts.len() == Actions::NUM_ELEMENTS);

        let mut rng = thread_rng();
        let max_iterations = 100000;

        for _ in 0..max_iterations {

            let action: Actions = qlearner.determine_learning_action(0, &mut rng);

            counts[action.to_index()] += 1.0;
        }

        // chi-squared should not exceed this for 95% confidence.
        let p_05 = 11.07;

        let expected_count = (max_iterations as f64) / (counts.len() as f64);

        let mut chi_sqr = 0.0f64;
        for count in &counts {
            let delta = count - expected_count;
            chi_sqr += (delta * delta) / expected_count;
        }

        println!("");
        println!(
            "north count = {}, ratio = {}",
            counts[Actions::North.to_index()],
            counts[Actions::North.to_index()] / expected_count
        );

        println!(
            "south count = {}, ratio = {}",
            counts[Actions::South.to_index()],
            counts[Actions::South.to_index()] / expected_count
        );

        println!(
            "east count = {}, ratio = {}",
            counts[Actions::East.to_index()],
            counts[Actions::East.to_index()] / expected_count
        );

        println!(
            "west count = {}, ratio = {}",
            counts[Actions::West.to_index()],
            counts[Actions::West.to_index()] / expected_count
        );

        println!(
            "pickup count = {}, ratio = {}",
            counts[Actions::PickUp.to_index()],
            counts[Actions::PickUp.to_index()] / expected_count
        );


        println!(
            "dropoff count = {}, ratio = {}",
            counts[Actions::DropOff.to_index()],
            counts[Actions::DropOff.to_index()] / expected_count
        );

        println!("chi-squared = {}, 95% confidence = {}", chi_sqr, p_05);

        assert!(chi_sqr < p_05);
    }
}
