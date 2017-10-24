
use rand::{Rng, thread_rng};


use taxi::state::State;
use taxi::actions::Actions;
use taxi::world::World;

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

            if let Some(passenger_index) =
                match state.get_passenger() {
                    Some(passenger_id) => world.get_fixed_index(passenger_id),
                    None => Some(self.num_passenger_states - 1),
                }
            {
                let taxi_pos = state.get_taxi();
                let taxi_index = (world.width * taxi_pos.y + taxi_pos.x) as usize;

                return Some(
                    destination_index * self.num_passenger_states * self.num_taxi_states +
                        passenger_index * self.num_taxi_states + taxi_index,
                );
            }
        }

        None
    }
}

pub struct QLearner {
    alpha: f64,
    gamma: f64,
    epsilon: f64,

    state_indexer: StateIndexer,
    qtable: Vec<[f64; Actions::NUM_ELEMENTS]>,
}

impl QLearner {
    pub fn new(world: &World) -> QLearner {

        let state_indexer = StateIndexer::new(&world);
        let num_states = state_indexer.num_states();
        let mut qtable = Vec::with_capacity(num_states);

        for _ in 0..num_states {
            qtable.push([0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64]);
        }

        QLearner {
            alpha: 1.0,
            gamma: 1.0,
            epsilon: 0.0,

            state_indexer,
            qtable,
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

            let action: Actions = rng.gen();
            state = state.apply_action(&world, action);
        }

        None

    }

    fn attempt(&self, world: &World, state: State, max_steps: usize) -> Attempt {
        Attempt::new(state, max_steps)
    }
}

#[cfg(test)]
mod test_qlearner {

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

        let world = World::build_from_str(&source_world).unwrap();

    }
}
