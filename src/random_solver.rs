
use rand::Rng;


use state::State;
use actions::Actions;
use world::World;

use runner::{Runner, Attempt};

#[derive(Default)]
pub struct RandomSolver {}

impl RandomSolver {
    pub fn new() -> RandomSolver {
        RandomSolver {}
    }
}

impl Runner for RandomSolver {
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

            let action: Actions = rng.gen();
            state = state.apply_action(world, action);
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

            let action: Actions = rng.gen();
            attempt.step(action);

            state = state.apply_action(world, action);
        }

        if state.at_destination() {
            attempt.succeeded();
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

            let action: Actions = rng.gen();
            state = state.apply_action(world, action);
        }

        state.at_destination()
    }
}
