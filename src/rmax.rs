use rand::Rng;


use state::State;
//use actions::Actions;
use world::World;

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
}


impl Runner for RMax {
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

            let next_action = rng.gen();
            state.apply_action(world, next_action);
        }

        None
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

            let next_action = rng.gen();
            attempt.step(next_action);
            state.apply_action(world, next_action);
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

            let next_action = rng.gen();
            state.apply_action(world, next_action);
        }
        state.at_destination()
    }

    fn report_training_result(&self, _world: &World) {}
}
