
use rand::{Rng, thread_rng};

use taxi::state::State;
use taxi::actions::Actions;


pub struct RandomSolver {
    pub iterations: u32,
    pub solution: Option<Vec<Actions>>,
}

impl RandomSolver {
    pub fn new(state: State, max_iterations: u32) -> RandomSolver {

        let mut rng = thread_rng();

        let mut iterations = 0;
        let mut applied_actions = Vec::new();

        let mut current_state = state;

        loop {
            if current_state.at_destination() {
                break RandomSolver {
                    iterations,
                    solution: Some(applied_actions),
                };
            }

            if iterations >= max_iterations {
                break RandomSolver {
                    iterations,
                    solution: None,
                };
            }

            iterations += 1;

            let action: Actions = rng.gen();

            applied_actions.push(action);
            current_state = current_state.apply_action(action);
        }
    }
}
