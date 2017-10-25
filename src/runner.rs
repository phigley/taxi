
use std::fmt;

use rand::thread_rng;

use state;
use state::State;
use world::World;
use actions::Actions;

#[derive(Debug)]
pub struct Probe {
    pub state: State,
    pub maximum_steps: usize,
}

impl Probe {
    pub fn new(state: State, maximum_steps: usize) -> Probe {
        Probe {
            state,
            maximum_steps,
        }
    }
}

pub trait Runner {
    fn learn(&mut self, world: &World, state: State, max_steps: usize) -> Option<usize>;
    fn attempt(&self, world: &World, state: State, max_steps: usize) -> Attempt;

    fn report_training_result(&self, _world: &World) {}
}

pub struct Attempt {
    pub initial_state: State,
    pub actions: Vec<Actions>,
    pub success: bool,
}

impl Attempt {
    pub fn new(initial_state: State, max_steps: usize) -> Attempt {
        Attempt {
            initial_state,
            actions: Vec::with_capacity(max_steps),
            success: false,
        }
    }

    pub fn step(&mut self, next_action: Actions) {
        self.actions.push(next_action);
    }

    pub fn succeeded(&mut self) {
        self.success = true;
    }
}

pub enum Error {
    BuildRandomState(state::Error),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        match *self {
            Error::BuildRandomState(ref state_error) => {
                write!(f, "Failed to build random state:\n{:?}", state_error)
            }
        }
    }
}



pub fn run_training_session<R: Runner>(
    world: &World,
    probes: &[Probe],
    max_trials: usize,
    max_steps: usize,
    runner: &mut R,
) -> Result<Option<usize>, Error> {

    let mut rng = thread_rng();

    let mut total_steps = 0;

    for _ in 0..max_trials {

        match State::build_random(world, &mut rng) {
            Err(state_error) => {
                return Err(Error::BuildRandomState(state_error));
            }

            Ok(state) => {
                if let Some(num_steps) = runner.learn(&world, state, max_steps) {
                    total_steps += num_steps;
                } else {
                    total_steps += max_steps;
                }
            }
        }

        let mut failed = false;
        for probe in probes {

            let attempt = runner.attempt(&world, probe.state.clone(), probe.maximum_steps);
            if !attempt.success {
                failed = true;
                break;
            }
        }

        if !failed {
            return Ok(Some(total_steps));
        }
    }

    Ok(None)
}
