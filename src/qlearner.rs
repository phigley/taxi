

use std::cmp;

use rand::Rng;
use float_cmp::ApproxOrdUlps;

use state::State;
use actions::Actions;
use world::World;
use state_indexer::StateIndexer;

use runner::{Runner, Attempt};


#[derive(Debug, Clone)]
pub struct QLearner {
    alpha: f64,
    gamma: f64,
    epsilon: f64,

    state_indexer: StateIndexer,
    qtable: Vec<[f64; Actions::NUM_ELEMENTS]>,
    show_table: bool,
}

impl QLearner {
    pub fn new(world: &World, alpha: f64, gamma: f64, epsilon: f64, show_table: bool) -> QLearner {

        let initial_q_value = world.max_reward() / (1.0 - gamma);

        let state_indexer = StateIndexer::new(world);
        let num_states = state_indexer.num_states();
        let qtable = vec![[initial_q_value; Actions::NUM_ELEMENTS]; num_states];

        QLearner {
            alpha,
            gamma,
            epsilon,

            state_indexer,
            qtable,
            show_table,
        }
    }

    fn determine_greedy_action<R: Rng>(&self, state_index: usize, rng: &mut R) -> Option<Actions> {
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
                match value.approx_cmp(&best_value, 2) {
                    cmp::Ordering::Greater => {
                        best_action = Actions::from_index(i);
                        best_value = *value;
                        num_found = 1;
                    }
                    cmp::Ordering::Equal => {
                        num_found += 1;
                        if rng.gen_range(0, num_found) == 0 {
                            best_action = Actions::from_index(i);
                        }
                    }
                    cmp::Ordering::Less => {}
                }
            }
        }

        best_action
    }

    fn determine_learning_action<R: Rng>(
        &self,
        state_index: usize,
        mut rng: &mut R,
    ) -> Option<Actions> {

        let nongreedy_roll = rng.gen_range(0.0f64, 1.0f64);

        if nongreedy_roll < self.epsilon {
            Actions::from_index(rng.gen_range(0, Actions::NUM_ELEMENTS))
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
    fn learn<R: Rng>(
        &mut self,
        world: &World,
        mut state: State,
        max_steps: usize,
        mut rng: &mut R,
    ) -> Option<usize> {

        for step in 0..max_steps {
            if state.at_destination() {
                return Some(step);
            }

            if let Some(state_index) = self.state_indexer.get_index(world, &state) {
                if let Some(next_action) = self.determine_learning_action(state_index, &mut rng) {
                    let (reward, next_state) = state.apply_action(world, next_action);

                    if let Some(next_state_index) =
                        self.state_indexer.get_index(world, &next_state)
                    {
                        self.apply_experience(state_index, next_action, next_state_index, reward);
                    } else {
                        return None;
                    }

                    state = next_state;
                } else {
                    return None;
                }
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
        mut rng: &mut R,
    ) -> Attempt {

        let mut attempt = Attempt::new(state, max_steps);

        for _ in 0..max_steps {
            if state.at_destination() {
                break;
            }

            if let Some(state_index) = self.state_indexer.get_index(world, &state) {

                if let Some(next_action) = self.determine_greedy_action(state_index, &mut rng) {

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
        mut rng: &mut R,
    ) -> bool {
        for _ in 0..max_steps {
            if state.at_destination() {
                return true;
            }

            if let Some(state_index) = self.state_indexer.get_index(world, &state) {
                if let Some(next_action) = self.determine_greedy_action(state_index, &mut rng) {
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

    fn report_training_result(&self, world: &World) {

        if self.show_table {
            println!();
            for (i, action_values) in self.qtable.iter().enumerate() {
                let state = self.state_indexer.get_state(world, i).unwrap();
                println!("{}", state.display(world));
                println!("{:?}", action_values);
            }
        }
    }
}

#[cfg(test)]
mod test_qlearner {

    use rand::thread_rng;

    use super::*;

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

        let mut qlearner = QLearner::new(&world, 1.0, 1.0, 0.0, false);

        let initial_index = qlearner
            .state_indexer
            .get_index(&world, &initial_state)
            .unwrap();

        let (reward, south_state) = initial_state.apply_action(&world, Actions::South);
        assert_eq!(expected_initial_str, south_state.display(&world));

        let south_index = qlearner
            .state_indexer
            .get_index(&world, &south_state)
            .unwrap();

        assert_eq!(south_index, initial_index);

        qlearner.apply_experience(initial_index, Actions::South, south_index, reward);

        let best_action = qlearner
            .determine_greedy_action(south_index, &mut rng)
            .unwrap();
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

        let qlearner = QLearner::new(&world, 1.0, 1.0, 0.0, false);

        let mut counts = vec![0.0f64; Actions::NUM_ELEMENTS];

        let mut rng = thread_rng();
        let max_iterations = 100000;

        for _ in 0..max_iterations {

            let action: Actions = qlearner.determine_greedy_action(0, &mut rng).unwrap();

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

        let qlearner = QLearner::new(&world, 1.0, 1.0, 0.0, false);

        let mut counts = vec![0.0f64; Actions::NUM_ELEMENTS];

        assert!(counts.len() == Actions::NUM_ELEMENTS);

        let mut rng = thread_rng();
        let max_iterations = 100000;

        for _ in 0..max_iterations {

            let action: Actions = qlearner.determine_learning_action(0, &mut rng).unwrap();

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
