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
use std::io;
use std::fmt;
use std::convert::From;

use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};

use termion::event;
use termion::input::TermRead;

use configuration::{Configuration, ReplayMode, InitialState};
use replay::Replay;

use random_solver::RandomSolver;

use taxi::state::State;
use taxi::world::World;

fn main() {

    if let Err(error) = run() {
        println!("{:?}", error);
    }
}

enum AppError {
    Configuration(configuration::Error),
    World(taxi::world::Error),
    InitialState(taxi::state::Error),
}

impl From<configuration::Error> for AppError {
    fn from(error: configuration::Error) -> Self {
        AppError::Configuration(error)
    }
}

impl From<taxi::world::Error> for AppError {
    fn from(error: taxi::world::Error) -> Self {
        AppError::World(error)
    }
}

impl From<taxi::state::Error> for AppError {
    fn from(error: taxi::state::Error) -> Self {
        AppError::InitialState(error)
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppError::Configuration(ref config_error) => {
                write!(f, "Failed to read configuration:\n{:?}", config_error)
            }
            AppError::World(ref world_error) => {
                write!(f, "Failed to build world:\n{:?}", world_error)
            }
            AppError::InitialState(ref state_error) => {
                write!(f, "Failed to build initial state:\n{:?}", state_error)
            }
        }
    }
}

fn run() -> Result<(), AppError> {

    let args: Vec<String> = env::args().collect();

    let config = if args.len() < 2 {
        Configuration::default()
    } else {
        Configuration::from_file(&args[1])?
    };

    let world = World::build_from_str(&config.world)?;
    let initial_states =
        build_initial_states(config.trials as usize, &config.initial_states, &world)?;

    execute_trials(&config, &world, &initial_states);

    Ok(())
}

fn build_initial_states(
    trials: usize,
    config_initial_states: &[InitialState],
    world: &World,
) -> Result<Vec<State>, AppError> {

    let mut initial_states = Vec::new();

    let num_config_initial_states = config_initial_states.len();

    if num_config_initial_states > 0 {

        for i in 0..trials {

            let initial_state = &config_initial_states[i % num_config_initial_states];

            let state = State::build(
                &world,
                initial_state.taxi_pos,
                initial_state.passenger_loc,
                initial_state.destination_loc,
            )?;

            initial_states.push(state);
        }

    } else {

        let mut rng = thread_rng();

        for _ in 0..trials {

            let state = State::build_random(&world, &mut rng)?;
            initial_states.push(state);
        }
    }

    Ok(initial_states)
}

fn execute_trials(config: &Configuration, world: &World, initial_states: &[State]) {

    let mut rng = thread_rng();
    let select_offset = Range::new(0, initial_states.len());

    let mut replay_result = None;

    let mut successes = Vec::new();

    for trial_num in 0..config.trials {

        let initial_state_offset = select_offset.ind_sample(&mut rng);
        let initial_state = initial_states[initial_state_offset];

        let result = RandomSolver::new(&world, initial_state.clone(), config.max_steps);

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
                        &world,
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
                            &world,
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
                            &world,
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
