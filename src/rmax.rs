use rand::Rng;


use state::State;
use actions::Actions;
use world::{World, ActionAffect};

use runner::{Runner, Attempt};



#[derive(Debug, Clone)]
pub struct RMax {
    movement_cost: f64,
    miss_passenger_cost: f64,
}

impl RMax {
    pub fn new() -> RMax {

        RMax {
            movement_cost: -1.0,
            miss_passenger_cost: -10.0,
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
}


impl Runner for RMax {
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

            let next_action = rng.gen();
            let _reward = self.determine_reward(world, &state, next_action);

            state = state.apply_action(world, next_action);
        }

        None
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

            let next_action = rng.gen();
            attempt.step(next_action);
            state = state.apply_action(world, next_action);
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

            let next_action = rng.gen();
            state = state.apply_action(world, next_action);
        }
        state.at_destination()
    }

    fn report_training_result(&self, world: &World) {}
}
