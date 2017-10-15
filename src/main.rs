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
use std::io;
use std::io::prelude::*;

use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};

use termion::event;
use termion::input::TermRead;

use configuration::{Configuration, ReplayMode};
use replay::Replay;

use random_solver::RandomSolver;

use taxi::state::State;
use taxi::world::World;

fn main() {

    let args: Vec<String> = env::args().collect();

    let config = if args.len() < 2 {
        Configuration::default()
    } else {

        let mut config_file =
            File::open(&args[1]).expect(&format!("Failed to open file '{}'", args[1]));

        let mut config_string = String::new();
        config_file.read_to_string(&mut config_string).expect(
            &format!("Failed to read file '{}'", args[1],),
        );

        match toml::from_str(&config_string) {
            Ok(result) => result,
            Err(err) => panic!("Failed to parse config file '{}' - {}", args[1], err),
        }
    };

    if config.initial_states.len() > 0 {

        match World::build_from_str(&config.initial_states[0]) {
            Err(msg) => {
                println!("Failed to build world: {}", msg);
                println!("Using source:");
                println!("{}", config.initial_states[0]);
            }
            Ok(w) => {

                let mut initial_states = Vec::new();

                for ref initial_state_str in &config.initial_states {

                    match State::build_from_str(initial_state_str, &w) {
                        Err(msg) => {
                            println!("Failed to build state: {}", msg);
                            println!("Using state:");
                            println!("{}", config.initial_states[0]);
                        }

                        Ok(initial_state) => initial_states.push(initial_state),
                    }
                }

                if initial_states.len() > 0 {
                    execute_trials(&config, &initial_states);
                } else {
                    println!("Failed to parse any initial_states, cannot execute trials.");
                }
            }
        }
    } else {
        println!("Found no initial_states, cannot run trials.");
    }


}


fn execute_trials(config: &Configuration, initial_states: &[State]) {

    let mut rng = thread_rng();
    let select_offset = Range::new(0, initial_states.len());

    let mut replay_result = None;

    let mut successes = Vec::new();

    for trial_num in 0..config.trials {

        let initial_state_offset = select_offset.ind_sample(&mut rng);
        let initial_state = initial_states[initial_state_offset];

        let result = RandomSolver::new(initial_state.clone(), config.max_steps);

        let num_steps = result.applied_actions.len();

        if result.solved {

            successes.push(num_steps as f64);

            println!(
                "{} - Solved {} after {} steps.",
                trial_num,
                initial_state_offset,
                num_steps
            );
        } else {
            println!(
                "{} - Failed {} after {} steps.",
                trial_num,
                initial_state_offset,
                num_steps
            );
        }

        match config.replay_mode {
            ReplayMode::None => (),
            ReplayMode::First => {
                if let None = replay_result {
                    replay_result = Some(Replay::new(
                        initial_state.clone(),
                        result.solved,
                        &result.applied_actions,
                    ));
                }
            }

            ReplayMode::FirstSuccess => {
                if result.solved {
                    if let None = replay_result {
                        replay_result = Some(Replay::new(
                            initial_state.clone(),
                            result.solved,
                            &result.applied_actions,
                        ));
                    }
                }
            }

            ReplayMode::FirstFailure => {
                if !result.solved {
                    if let None = replay_result {
                        replay_result = Some(Replay::new(
                            initial_state.clone(),
                            result.solved,
                            &result.applied_actions,
                        ));
                    }
                }
            }
        }
    }

    if config.trials > 0 {

        let success_percent = (successes.len() as f64) / (config.trials as f64);

        println!(
            "Averaged {:.1} % success.  Failure at {} steps.",
            success_percent * 100.0,
            config.max_steps
        );

        if successes.len() > 1 {

            let mut average = 0.0f64;
            let mut variance_sum = 0.0f64;
            let mut count = 0.0f64;

            for s in successes {

                let old_average = average;

                count += 1.0;
                average += (s - average) / count;

                variance_sum += (s - old_average) * (s - average);
            }

            let sample_stddev_sqr = variance_sum / (count - 1.0);
            let sample_stddev = sample_stddev_sqr.sqrt();

            println!("Avg steps = {:.2}  Std Dev = {:.2}", average, sample_stddev);
        }

    }

    if let Some(replay) = replay_result {

        if let Some(_) = wait_for_input() {
            if let Err(error) = replay.run() {
                println!("IO error : {:?}", error);
            }
        }
    }
}

fn wait_for_input() -> Option<()> {
    println!("Press Enter to see replay.  q to exit.");

    loop {
        for c in io::stdin().keys() {

            match c {
                Ok(evt) => {
                    match evt {
                        event::Key::Char('q') |
                        event::Key::Char('Q') => return None,
                        event::Key::Char('\n') => return Some(()),
                        _ => (),
                    }
                }
                Err(_) => return None,
            }
        }
    }
}
