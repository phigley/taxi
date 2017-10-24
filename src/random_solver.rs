
use rand::{Rng, thread_rng};


use taxi::state::State;
use taxi::actions::Actions;
use taxi::world::World;

use runner::{Runner, Attempt};

pub struct RandomSolver {}

impl RandomSolver {
    pub fn new() -> RandomSolver {
        RandomSolver {}
    }
}

impl Runner for RandomSolver {
    fn learn(&mut self, world: &World, state: State, max_steps: usize) -> Option<usize> {

        let mut rng = thread_rng();


        let mut current_state = state;

        for step in 0..max_steps {
            if current_state.at_destination() {
                return Some(step);
            }

            let action: Actions = rng.gen();
            current_state = current_state.apply_action(&world, action);
        }

        None
    }

    fn attempt(&self, world: &World, mut state: State, max_steps: usize) -> Attempt {

        let mut rng = thread_rng();

        let mut attempt = Attempt::new(state.clone(), max_steps);

        for _ in 0..max_steps {
            if state.at_destination() {
                attempt.succeeded();
                break;
            }

            let action: Actions = rng.gen();
            attempt.step(action);

            state = state.apply_action(&world, action);
        }

        attempt
    }
}
