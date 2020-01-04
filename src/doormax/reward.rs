use crate::actions::Actions;
use crate::state::State;
use crate::world::World;

#[derive(Debug, Clone)]
pub struct Rewards {
    reward_starts: [usize; Actions::NUM_ELEMENTS],
    occurences: Vec<f64>,
    rewards: Vec<f64>,

    known_count: f64,
}

impl Rewards {
    pub fn new(world: &World, known_count: f64) -> Rewards {
        let num_total_reward_parents = total_reward_parents(world);
        let mut next_reward_index = 0;
        let mut reward_starts = [0; Actions::NUM_ELEMENTS];

        for (action_index, reward_start) in reward_starts.iter_mut().enumerate() {
            let action = Actions::from_index(action_index).unwrap();

            *reward_start = next_reward_index;
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

    pub fn apply_experience(&mut self, reward: f64, world: &World, state: &State, action: Actions) {
        let x_index = state.get_taxi().x as usize;
        let y_index = state.get_taxi().y as usize;

        if let Some(passenger_index) = generate_passenger_index(world, state) {
            if let Some(destination_index) = generate_destination_index(world, state) {
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
        }
    }

    pub fn get_reward(&self, world: &World, state: &State, action: Actions) -> Option<f64> {
        let x_index = state.get_taxi().x as usize;
        let y_index = state.get_taxi().y as usize;

        let passenger_index = generate_passenger_index(world, state)?;
        let destination_index = generate_destination_index(world, state)?;

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

    num_taxi_values * 4
        + num_passenger_values * num_taxi_values
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

fn generate_passenger_index(world: &World, state: &State) -> Option<usize> {
    match state.get_passenger() {
        None => Some(0),
        Some(passenger_id) => world.get_fixed_index(passenger_id).map(|i| i + 1),
    }
}

fn generate_destination_index(world: &World, state: &State) -> Option<usize> {
    world.get_fixed_index(state.get_destination())
}
