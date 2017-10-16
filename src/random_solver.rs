
use rand::{Rng, thread_rng};

use taxi::state::State;
use taxi::actions::Actions;
use taxi::world::World;

pub struct RandomSolver {
    pub iterations: u32,
    pub solved: bool,
    pub applied_actions: Vec<Actions>,
}

impl RandomSolver {
    pub fn new(world: &World, state: State, max_iterations: u32) -> RandomSolver {

        let mut rng = thread_rng();

        let mut iterations = 0;
        let mut applied_actions = Vec::new();

        let mut current_state = state;

        loop {
            if current_state.at_destination() {
                break RandomSolver {
                    iterations,
                    solved: true,
                    applied_actions,
                };
            }

            if iterations >= max_iterations {
                break RandomSolver {
                    iterations,
                    solved: false,
                    applied_actions,
                };
            }

            iterations += 1;

            let action: Actions = rng.gen();

            applied_actions.push(action);
            current_state = current_state.apply_action(&world, action);
        }
    }
}
