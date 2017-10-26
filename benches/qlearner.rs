#![feature(test)]

extern crate test;
extern crate taxi;
extern crate rand;

use rand::{SeedableRng, StdRng};

use taxi::world::World;
use taxi::state::State;
use taxi::runner::{run_training_session, Probe};
use taxi::qlearner::QLearner;

#[bench]
fn bench_run_training_session(b: &mut test::Bencher) {

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

    let seed: &[_] = &[1, 2, 3, 4];

    let world = World::build_from_str(world_str).unwrap();

    let probes = [
        Probe::new(State::build(&world, (2, 2), Some('Y'), 'R').unwrap(), 10),
        Probe::new(State::build(&world, (2, 2), Some('Y'), 'G').unwrap(), 14),
        Probe::new(State::build(&world, (2, 2), Some('Y'), 'B').unwrap(), 13),
        Probe::new(State::build(&world, (2, 2), Some('R'), 'B').unwrap(), 13),
        Probe::new(State::build(&world, (2, 2), Some('Y'), 'R').unwrap(), 6),
        Probe::new(State::build(&world, (2, 2), Some('B'), 'G').unwrap(), 13),
    ];

    b.iter(|| {
        let mut qlearner = QLearner::new(&world, 0.1, 0.3, 0.6, false);
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        run_training_session(&world, &probes, 1000, 100, &mut qlearner, &mut rng)
    })
}
