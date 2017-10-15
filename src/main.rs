extern crate rand;
extern crate tui;
extern crate termion;
extern crate taxi;

mod replay;


use rand::Rng;

use replay::Replay;

use taxi::state::State;
use taxi::world::World;
use taxi::actions::Actions;

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

struct RandomSolver {
    iterations: u32,
    solution: Option<Vec<Actions>>,
}

impl RandomSolver {
    fn new(state: State, max_iterations: u32) -> RandomSolver {

        let mut rng = rand::thread_rng();

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
