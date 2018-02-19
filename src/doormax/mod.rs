mod term;
mod condition;
mod hypothesis;
mod effect;
mod condition_learner;
mod effect_learner;
mod celearner;

use rand::Rng;

use state::State;
use world::World;

use runner::{Attempt, Runner};

#[derive(Debug, Clone)]
pub struct DoorMaxParams {}

#[derive(Debug, Clone)]
pub struct DoorMax {
    params: DoorMaxParams,
}

impl DoorMax {
    pub fn new(_world: &World) -> Self {
        let params = DoorMaxParams {};

        DoorMax { params }
    }
}

impl Runner for DoorMax {
    fn learn<R: Rng>(
        &mut self,
        _world: &World,
        _state: State,
        _max_steps: usize,
        _rng: &mut R,
    ) -> Option<usize> {
        None
    }

    fn attempt<R: Rng>(
        &self,
        _world: &World,
        state: State,
        max_steps: usize,
        _rng: &mut R,
    ) -> Attempt {
        let attempt = Attempt::new(state, max_steps);

        attempt
    }

    fn solves<R: Rng>(
        &self,
        _world: &World,
        state: State,
        _max_steps: usize,
        _rng: &mut R,
    ) -> bool {
        state.at_destination()
    }
}
