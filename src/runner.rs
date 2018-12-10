use std::fmt;

use rand::Rng;

use crate::actions::Actions;
use crate::state;
use crate::state::State;
use crate::world::World;

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
    fn learn<R: Rng>(
        &mut self,
        world: &World,
        state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Option<usize>;
    fn attempt<R: Rng>(
        &self,
        world: &World,
        state: State,
        max_steps: usize,
        rng: &mut R,
    ) -> Attempt;

    fn solves<R: Rng>(&self, world: &World, state: State, max_steps: usize, rng: &mut R) -> bool;

    fn report_training_result(&self, _world: &World, _steps: Option<usize>) {}
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::BuildRandomState(ref state_error) => {
                write!(f, "Failed to build random state:\n{:?}", state_error)
            }
        }
    }
}

pub fn run_training_session<Rnr, R>(
    world: &World,
    probes: &[Probe],
    max_trials: usize,
    max_steps: usize,
    runner: &mut Rnr,
    mut rng: &mut R,
) -> Result<Option<usize>, Error>
where
    Rnr: Runner,
    R: Rng,
{
    let mut total_steps = 0;

    for _ in 0..max_trials {
        match State::build_random(world, &mut rng) {
            Err(state_error) => {
                return Err(Error::BuildRandomState(state_error));
            }

            Ok(state) => {
                if let Some(num_steps) = runner.learn(world, state, max_steps, &mut rng) {
                    total_steps += num_steps;
                } else {
                    total_steps += max_steps;
                }
            }
        }

        let probes_passed = probes
            .iter()
            .all(|probe| runner.solves(world, probe.state, probe.maximum_steps, &mut rng));

        if probes_passed {
            return Ok(Some(total_steps));
        }
    }

    Ok(None)
}
