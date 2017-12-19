

use state::State;
use actions::Actions;
use world::World;

use maxq::MaxQParams;

#[derive(Debug, Clone)]
pub struct PrimitiveNode {
    action: Actions,
    values: Vec<f64>,
}

impl PrimitiveNode {
    pub fn new(action: Actions, initial_q_value: f64) -> PrimitiveNode {
        let num_values = match action {
            Actions::PickUp | Actions::DropOff => 2,
            Actions::North | Actions::South | Actions::East | Actions::West => 1,
        };

        PrimitiveNode {
            action,
            values: vec![initial_q_value; num_values],
        }
    }

    fn get_value_index(&self, world: &World, state: &State) -> usize {
        match self.action {
            // Pick-up has only 2 results, taxi is at passenger or not.
            Actions::PickUp => {
                match state.get_passenger() {
                    Some(id) if world.get_fixed_position(id) == Some(state.get_taxi()) => 0,
                    _ => 1,
                }
            }

            // Drop-off has only 2 results, passenger is in taxi and at destination or not.
            Actions::DropOff => {
                match world.get_fixed_id(state.get_taxi()) {
                    Some(id) if state.get_passenger() == None && id == state.get_destination() => 0,
                    _ => 1,
                }
            }

            // reward for directional movement is independent of taxi position
            Actions::North | Actions::South | Actions::East | Actions::West => 0,
        }
    }

    pub fn evaluate(&self, world: &World, state: &State) -> (f64, Actions) {
        let value_index = self.get_value_index(world, state);
        (self.values[value_index], self.action)
    }

    pub fn apply_experience(
        &mut self,
        params: &MaxQParams,
        world: &World,
        state: &State,
        reward: f64,
        next_state: &State,
    ) {

        let value_index = self.get_value_index(world, &state);

        self.values[value_index] *= 1.0 - params.alpha;
        self.values[value_index] += params.alpha * reward;
    }

    pub fn get_action(&self) -> Actions {
        self.action
    }

    pub fn build_nodes(initial_q_value: f64) -> Vec<PrimitiveNode> {

        let mut result = Vec::with_capacity(Actions::NUM_ELEMENTS);

        for action_index in 0..Actions::NUM_ELEMENTS {
            let action = Actions::from_index(action_index).unwrap();
            result.push(Self::new(action, initial_q_value));
        }

        result
    }
}
