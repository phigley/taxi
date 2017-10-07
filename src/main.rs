#[cfg(test)]
#[macro_use]
extern crate assert_matches;

extern crate rand;

mod position;
mod world;
mod state;
mod actions;

use rand::Rng;

use state::State;
use world::World;
use actions::Actions;

fn main() {
    let source = "\
        d....\n\
        ...p.\n\
        .....\n\
        #t...\n\
        .....\n\
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

                Ok(state) => {
                    let result = RandomSolver::new(state.clone(), 200);

                    if let Some(actions) = result.solution {
                        println!(
                            "Found solution in {} steps, it is {} steps long.",
                            result.iterations,
                            actions.len()
                        );

                        let mut display_state = state.clone();
                        println!("-------------- {} -----------------", 0);
                        show_state(&display_state);

                        for (i, a) in actions.into_iter().enumerate() {
                            display_state = display_state.apply_action(a);
                            println!("-------------- {} -----------------", i + 1);
                            show_state(&display_state);
                        }


                    } else {
                        println!("Failed to find solution in {} steps.", result.iterations);
                    }
                }
            }
        }
    }
}


fn show_state(state: &State) {
    match state.display() {
        Err(msg) => println!("Failed to display: {}", msg),
        Ok(state_str) => println!("{}", state_str),
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
