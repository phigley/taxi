use state::State;
use world::World;

#[derive(Debug, Clone, Copy)]
pub struct StateIndexer {
    num_taxi_states: usize,
    num_passenger_states: usize,
    num_destination_states: usize,
}

impl StateIndexer {
    pub fn new(world: &World) -> StateIndexer {
        let num_taxi_states = (world.width * world.height) as usize;
        let num_destination_states = world.num_fixed_positions();
        let num_passenger_states = num_destination_states + 1;

        StateIndexer {
            num_taxi_states,
            num_passenger_states,
            num_destination_states,
        }
    }

    pub fn num_states(&self) -> usize {
        self.num_taxi_states * self.num_passenger_states * self.num_destination_states
    }

    pub fn get_index(&self, world: &World, state: &State) -> Option<usize> {
        if let Some(destination_index) = world.get_fixed_index(state.get_destination()) {
            let mut result = destination_index;

            if let Some(passenger_index) = match state.get_passenger() {
                Some(passenger_id) => world.get_fixed_index(passenger_id),
                None => Some(self.num_passenger_states - 1),
            } {
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

    pub fn get_state(&self, world: &World, mut state_index: usize) -> Option<State> {
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
                world,
                (taxi_x as i32, taxi_y as i32),
                passenger,
                destination,
            )
            .ok()
        } else {
            None
        }
    }
}
