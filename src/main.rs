#[macro_use]
extern crate serde_derive;

extern crate rand;
extern crate tui;
extern crate termion;
extern crate toml;
extern crate taxi;

mod configuration;
mod replay;
mod random_solver;

use std::env;
use std::fs::File;
use std::io::prelude::*;

use configuration::Configuration;
use replay::Replay;

use random_solver::RandomSolver;

use taxi::state::State;
use taxi::world::World;

fn main() {

    let args: Vec<String> = env::args().collect();

    let input = if args.len() < 2 {
        Configuration::default()
    } else {

        let mut config_file =
            File::open(&args[1]).expect(&format!("Failed to open file '{}'", args[1]));

        let mut config_string = String::new();
        config_file.read_to_string(&mut config_string).expect(
            &format!("Failed to read file '{}'", args[1],),
        );

        toml::from_str(&config_string).unwrap()
    };

    match World::build_from_str(&input.initial_state) {
        Err(msg) => {
            println!("Failed to build world: {}", msg);
            println!("Using source:");
            println!("{}", input.initial_state);
        }
        Ok(w) => {
            match State::build_from_str(&input.initial_state, &w) {
                Err(msg) => {
                    println!("Failed to build state: {}", msg);
                    println!("Using state:");
                    println!("{}", input.initial_state);
                }

                Ok(initial_state) => {
                    let result = RandomSolver::new(initial_state.clone(), input.max_steps);

                    if let Some(actions) = result.solution {

                        let replay = Replay::new(initial_state, &actions);

                        if let Err(error) = replay.run() {
                            println!("IO error : {:?}", error);
                        }

                    } else {
                        println!("{}", input.initial_state);
                        println!("Failed to find solution in {} steps.", result.iterations);
                    }
                }
            }
        }
    }
}
