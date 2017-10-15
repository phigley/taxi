extern crate rand;
extern crate tui;
extern crate termion;
extern crate taxi;

mod replay;
mod random_solver;


use replay::Replay;

use random_solver::RandomSolver;

use taxi::state::State;
use taxi::world::World;

fn main() {
    let source = "\
        ┌───┬─────┐\n\
        │d .│. . .│\n\
        │   │     │\n\
        │. .│. . .│\n\
        │         │\n\
        │. . t . .│\n\
        │         │\n\
        │.│. .│. .│\n\
        │ │   │   │\n\
        │.│. .│p .│\n\
        └─┴───┴───┘\n\
        ";

    match World::build_from_str(source) {
        Err(msg) => {
            println!("Failed to build world: {}", msg);
            println!("Using source:");
            println!("{}", source);
        }
        Ok(w) => {
            match State::build_from_str(source, &w) {
                Err(msg) => {
                    println!("Failed to build state: {}", msg);
                    println!("Using state:");
                    println!("{}", source);
                }

                Ok(initial_state) => {
                    let result = RandomSolver::new(initial_state.clone(), 500);

                    if let Some(actions) = result.solution {

                        let replay = Replay::new(initial_state, &actions);

                        if let Err(error) = replay.run() {
                            println!("IO error : {:?}", error);
                        }

                    } else {
                        println!("Failed to find solution in {} steps.", result.iterations);
                    }
                }
            }
        }
    }
}
