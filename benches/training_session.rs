#![feature(test)]

extern crate test;
extern crate taxi;
extern crate rand;

use rand::Isaac64Rng;

use taxi::world::World;
use taxi::state::State;
use taxi::runner::{run_training_session, Probe};
use taxi::qlearner::QLearner;
use taxi::rmax::RMax;

struct SessionData {
    world: World,
    probes: Vec<Probe>,
}

impl Default for SessionData {
    fn default() -> SessionData {

        let world_str = "\
        ┌───┬─────┐\n\
        │R .│. . G│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . . . .│\n\
        │         │\n\
        │.│. .│. .│\n\
        │ │   │   │\n\
        │Y│. .│B .│\n\
        └─┴───┴───┘\n\
        ";

        let world = World::build_from_str(world_str).unwrap();

        let probes = vec![
            Probe::new(State::build(&world, (2, 2), Some('Y'), 'R').unwrap(), 10),
            Probe::new(State::build(&world, (2, 2), Some('Y'), 'G').unwrap(), 14),
            Probe::new(State::build(&world, (2, 2), Some('Y'), 'B').unwrap(), 13),
            Probe::new(State::build(&world, (2, 2), Some('R'), 'B').unwrap(), 13),
            Probe::new(State::build(&world, (2, 2), Some('Y'), 'R').unwrap(), 6),
            Probe::new(State::build(&world, (2, 2), Some('B'), 'G').unwrap(), 13),
        ];

        SessionData { world, probes }
    }
}

#[bench]
fn qlearner(b: &mut test::Bencher) {

    let data = SessionData::default();

    b.iter(|| {
        let mut qlearner = QLearner::new(&data.world, 0.1, 0.3, 0.6, false);
        let mut rng = Isaac64Rng::new_unseeded();

        run_training_session(&data.world, &data.probes, 1, 100, &mut qlearner, &mut rng)
    })
}

#[bench]
fn rmax(b: &mut test::Bencher) {

    let data = SessionData::default();

    b.iter(|| {
        let mut qlearner = RMax::new(&data.world, 0.3, 1.0, 1.0e-6);
        let mut rng = Isaac64Rng::new_unseeded();

        run_training_session(&data.world, &data.probes, 1, 100, &mut qlearner, &mut rng)
    })
}
