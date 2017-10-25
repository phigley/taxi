#[macro_use]
extern crate serde_derive;

extern crate rand;
extern crate tui;
extern crate termion;
extern crate toml;
extern crate taxi;

mod configuration;
mod replay;

use std::env;
use std::io;
use std::fmt;

use termion::event;
use termion::input::TermRead;

use configuration::{Configuration, SolverChoice};
use replay::Replay;

use taxi::state::State;
use taxi::world::World;
use taxi::distribution::MeasureDistribution;

use taxi::runner::{run_training_session, Probe, Runner};
use taxi::random_solver::RandomSolver;
use taxi::qlearner::QLearner;

fn main() {

    if let Err(error) = run() {
        println!("{:?}", error);
    }
}

enum AppError {
    Configuration(configuration::Error),
    World(taxi::world::Error),
    BuildProbes(taxi::state::Error),
    Runner(taxi::runner::Error),
    ReplayTraining(taxi::runner::Error),
    ReplayState(taxi::state::Error),
    Replay(io::Error),
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
            AppError::BuildProbes(ref state_error) => {
                write!(f, "Failed to build probe state:\n{:?}", state_error)
            }
            AppError::Runner(ref runner_error) => {
                write!(f, "Failed to run trial:\n{:?}", runner_error)
            }
            AppError::ReplayTraining(ref runner_error) => {
                write!(f, "Failed to run training for replay:\n{:?}", runner_error)
            }
            AppError::ReplayState(ref state_error) => {
                write!(f, "Failed to build replay state:\n{:?}", state_error)
            }
            AppError::Replay(ref replay_error) => {
                write!(f, "Failed to replay:\n{:?}", replay_error)
            }
        }
    }
}

fn run() -> Result<(), AppError> {

    let args: Vec<String> = env::args().collect();

    let config = if args.len() < 2 {
        Configuration::default()
    } else {
        Configuration::from_file(&args[1]).map_err(
            AppError::Configuration,
        )?
    };

    let world = World::build_from_str(&config.world).map_err(
        AppError::World,
    )?;
    let probes = build_probes(&config, &world)?;

    if config.sessions > 0 {
        if let Some(_) = config.random_solver {
            let stats = gather_stats(
                || RandomSolver::new(),
                &world,
                &probes,
                config.sessions,
                config.max_trials,
                config.max_trial_steps,
            )?;

            let (avg_steps, stddev_steps) = stats.distribution.get_distribution();

            println!(
                "{:?} - finished {} sessions in {} average steps with stddev of {}.",
                SolverChoice::Random,
                stats.completed,
                avg_steps,
                stddev_steps
            );
        };

        if let Some(qlearner_config) = config.q_learner.as_ref() {
            let stats = gather_stats(
                || {
                    QLearner::new(
                        &world,
                        qlearner_config.alpha,
                        qlearner_config.gamma,
                        qlearner_config.epsilon,
                        qlearner_config.show_table,
                    )
                },
                &world,
                &probes,
                config.sessions,
                config.max_trials,
                config.max_trial_steps,
            )?;

            let (avg_steps, stddev_steps) = stats.distribution.get_distribution();

            println!(
                "{:?} - finished {} sessions in {} average steps with stddev of {}.",
                SolverChoice::QLearner,
                stats.completed,
                avg_steps,
                stddev_steps
            );
        };
    }

    if let Some(replay_config) = config.replay.as_ref() {

        match replay_config.solver {
            SolverChoice::Random => {
                if let Some(_) = config.random_solver {
                    run_replay(
                        &mut RandomSolver::new(),
                        &replay_config,
                        &world,
                        &probes,
                        config.max_trials,
                        config.max_trial_steps,
                    )?
                } else {
                    println!(
                        "Attempting to replay {:?} solver with out a valid configuration \
                        for that solver.",
                        replay_config.solver
                    );
                }
            }
            SolverChoice::QLearner => {
                if let Some(qlearner_config) = config.q_learner.as_ref() {
                    run_replay(
                        &mut QLearner::new(
                            &world,
                            qlearner_config.alpha,
                            qlearner_config.gamma,
                            qlearner_config.epsilon,
                            qlearner_config.show_table,
                        ),
                        &replay_config,
                        &world,
                        &probes,
                        config.max_trials,
                        config.max_trial_steps,
                    )?
                } else {
                    println!(
                        "Attempting to replay {:?} solver with out a valid configuration \
                        for that solver.",
                        replay_config.solver
                    );
                }
            }
        };

    }

    Ok(())
}


fn build_probes(config: &Configuration, world: &World) -> Result<Vec<Probe>, AppError> {

    let mut probes = Vec::new();

    for probe_config in &config.probes {
        let state = State::build(
            &world,
            probe_config.taxi_pos,
            probe_config.passenger_loc,
            probe_config.destination_loc,
        ).map_err(AppError::BuildProbes)?;

        probes.push(Probe::new(state, probe_config.max_steps));
    }

    Ok(probes)
}

struct Stats {
    distribution: MeasureDistribution,
    completed: usize,
}

fn gather_stats<B, R>(
    builder: B,
    world: &World,
    probes: &[Probe],
    sessions: usize,
    max_trials: usize,
    max_trial_steps: usize,
) -> Result<Stats, AppError>
where
    B: Fn() -> R,
    R: Runner,
{
    let mut distribution = MeasureDistribution::new();
    let mut completed = 0;

    for s in 0..sessions {

        let mut solver = builder();

        if let Some(num_steps) = run_training_session(
            &world,
            &probes,
            max_trials,
            max_trial_steps,
            &mut solver,
        ).map_err(AppError::Runner)?
        {
            distribution.add_value(num_steps as f64);
            completed += 1;
        } else {
            println!("Failed session {}.", s);
        }

        solver.report_training_result(&world);

    }

    Ok(Stats {
        distribution,
        completed,
    })
}

fn run_replay<R>(
    solver: &mut R,
    replay_config: &configuration::Replay,
    world: &World,
    probes: &[Probe],
    max_trials: usize,
    max_trial_steps: usize,
) -> Result<(), AppError>
where
    R: Runner,
{
    run_training_session(&world, &probes, max_trials, max_trial_steps, solver)
        .map_err(AppError::ReplayTraining)?;

    if let Some(_) = wait_for_input() {
        let replay_state = State::build(
            &world,
            replay_config.taxi_pos,
            replay_config.passenger_loc,
            replay_config.destination_loc,
        ).map_err(AppError::ReplayState)?;

        let attempt = solver.attempt(&world, replay_state, replay_config.max_steps);

        let replay = Replay::new(&world, attempt);

        replay.run().map_err(AppError::Replay)?;

    }

    Ok(())
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
