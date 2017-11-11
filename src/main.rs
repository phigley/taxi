#[macro_use]
extern crate serde_derive;

extern crate rand;
extern crate tui;
extern crate termion;
extern crate toml;
extern crate rayon;

extern crate taxi;

mod configuration;
mod replay;

use std::env;
use std::io;
use std::fmt;

use rand::{Rng, thread_rng};

use termion::event;
use termion::input::TermRead;

use rayon::prelude::*;

use configuration::{Configuration, SolverChoice};
use replay::Replay;

use taxi::state::State;
use taxi::world::World;
use taxi::distribution::MeasureDistribution;

use taxi::runner::{run_training_session, Probe, Runner};
use taxi::random_solver::RandomSolver;
use taxi::qlearner::QLearner;
use taxi::rmax::RMax;
use taxi::factoredrmax::FactoredRMax;

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
    ReplayRunnerNotConfigured(SolverChoice),
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
            AppError::ReplayRunnerNotConfigured(ref runner_type) => {
                write!(
                    f,
                    "Attempting to replay {:?} solver with out a valid configuration \
                        for that solver.",
                    runner_type
                )
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

    let mut rng = thread_rng();

    let world = World::build_from_str(&config.world).map_err(
        AppError::World,
    )?;
    let probes = build_probes(&config, &world)?;

    if config.sessions > 0 {

        if config.random_solver.is_some() {
            gather_and_report_stats(
                RandomSolver::new,
                SolverChoice::Random,
                &world,
                &probes,
                &config,
            )?;
        };

        if let Some(qlearner_config) = config.q_learner.as_ref() {
            gather_and_report_stats(
                || {
                    QLearner::new(
                        &world,
                        qlearner_config.alpha,
                        qlearner_config.gamma,
                        qlearner_config.epsilon,
                        qlearner_config.show_table,
                    )
                },
                SolverChoice::QLearner,
                &world,
                &probes,
                &config,
            )?;
        };

        if let Some(rmax_config) = config.r_max.as_ref() {
            gather_and_report_stats(
                || {
                    RMax::new(
                        &world,
                        rmax_config.gamma,
                        rmax_config.known_count,
                        rmax_config.error_delta,
                    )
                },
                SolverChoice::RMax,
                &world,
                &probes,
                &config,
            )?;
        };

        if let Some(factored_rmax_config) = config.factored_r_max.as_ref() {
            gather_and_report_stats(
                || {
                    FactoredRMax::new(
                        &world,
                        factored_rmax_config.gamma,
                        factored_rmax_config.known_count,
                        factored_rmax_config.error_delta,
                    )
                },
                SolverChoice::FactoredRMax,
                &world,
                &probes,
                &config,
            )?;
        };
    }

    if let Some(replay_config) = config.replay.as_ref() {

        match replay_config.solver {
            SolverChoice::Random => {
                if config.random_solver.is_some() {
                    run_replay(
                        &mut RandomSolver::new(),
                        replay_config,
                        &world,
                        &probes,
                        config.max_trials,
                        config.max_trial_steps,
                        &mut rng,
                    )?
                } else {
                    return Err(AppError::ReplayRunnerNotConfigured(replay_config.solver));
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
                        replay_config,
                        &world,
                        &probes,
                        config.max_trials,
                        config.max_trial_steps,
                        &mut rng,
                    )?
                } else {
                    return Err(AppError::ReplayRunnerNotConfigured(replay_config.solver));
                }
            }

            SolverChoice::RMax => {
                if let Some(rmax_config) = config.r_max.as_ref() {
                    run_replay(
                        &mut RMax::new(
                            &world,
                            rmax_config.gamma,
                            rmax_config.known_count,
                            rmax_config.error_delta,
                        ),
                        replay_config,
                        &world,
                        &probes,
                        config.max_trials,
                        config.max_trial_steps,
                        &mut rng,
                    )?
                } else {
                    return Err(AppError::ReplayRunnerNotConfigured(replay_config.solver));
                }
            }
            SolverChoice::FactoredRMax => {
                if let Some(factored_rmax_config) = config.factored_r_max.as_ref() {
                    run_replay(
                        &mut FactoredRMax::new(
                            &world,
                            factored_rmax_config.gamma,
                            factored_rmax_config.known_count,
                            factored_rmax_config.error_delta,
                        ),
                        replay_config,
                        &world,
                        &probes,
                        config.max_trials,
                        config.max_trial_steps,
                        &mut rng,
                    )?
                } else {
                    return Err(AppError::ReplayRunnerNotConfigured(replay_config.solver));
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
            world,
            probe_config.taxi_pos,
            probe_config.passenger_loc,
            probe_config.destination_loc,
        ).map_err(AppError::BuildProbes)?;

        probes.push(Probe::new(state, probe_config.max_steps));
    }

    Ok(probes)
}


#[derive(Default)]
struct Stats {
    distribution: MeasureDistribution,
}

fn gather_and_report_stats<B, Rnr>(
    builder: B,
    solver_choice: SolverChoice,
    world: &World,
    probes: &[Probe],
    config: &Configuration,
) -> Result<(), AppError>
where
    B: Fn() -> Rnr + Sync,
    Rnr: Runner + Sync,
{
    let stats = gather_stats(
        builder,
        world,
        probes,
        config.sessions,
        config.max_trials,
        config.max_trial_steps,
    )?;

    let (avg_steps, stddev_steps) = stats.distribution.get_distribution();

    println!(
        "{:?} - finished {} sessions in {} average steps with stddev of {}.",
        solver_choice,
        stats.distribution.get_count() as usize,
        avg_steps,
        stddev_steps
    );

    Ok(())
}

fn gather_stats<B, Rnr>(
    builder: B,
    world: &World,
    probes: &[Probe],
    sessions: usize,
    max_trials: usize,
    max_trial_steps: usize,
) -> Result<Stats, AppError>
where
    B: Fn() -> Rnr + Sync,
    Rnr: Runner + Sync,
{
    let session_ids: Vec<usize> = (0..sessions).collect();

    session_ids
        .par_iter()
        .fold(|| Ok(Stats::default()), |current_result,
         session_number|
         -> Result<Stats, AppError> {

            current_result.and_then(|mut stats| {

                let mut solver = builder();
                let mut rng = thread_rng();

                let training_step_count = run_training_session(
                    world,
                    probes,
                    max_trials,
                    max_trial_steps,
                    &mut solver,
                    &mut rng,
                ).map_err(AppError::Runner)?;

                match training_step_count {
                    Some(num_steps) => {
                        stats.distribution.add_value(num_steps as f64);
                    }
                    None => {
                        println!("Failed session {}.", session_number);
                    }
                };

                // This may overlap with other reports, should we guard with a mutex?
                solver.report_training_result(world);

                Ok(stats)
            })
        })
        .reduce(|| Ok(Stats::default()), |result_a: Result<
            Stats,
            AppError,
        >,
         result_b: Result<
            Stats,
            AppError,
        >|
         -> Result<Stats, AppError> {

            result_a.and_then(|mut stats_a| {
                result_b.and_then(|stats_b| {
                    stats_a.distribution.add_distribution(&stats_b.distribution);
                    Ok(stats_a)
                })
            })

        })
}

fn run_replay<Rnr, R>(
    solver: &mut Rnr,
    replay_config: &configuration::Replay,
    world: &World,
    probes: &[Probe],
    max_trials: usize,
    max_trial_steps: usize,
    mut rng: &mut R,
) -> Result<(), AppError>
where
    Rnr: Runner,
    R: Rng,
{
    run_training_session(world, probes, max_trials, max_trial_steps, solver, &mut rng)
        .map_err(AppError::ReplayTraining)?;

    if wait_for_input().is_some() {
        let replay_state = State::build(
            world,
            replay_config.taxi_pos,
            replay_config.passenger_loc,
            replay_config.destination_loc,
        ).map_err(AppError::ReplayState)?;

        let attempt = solver.attempt(world, replay_state, replay_config.max_steps, &mut rng);

        let replay = Replay::new(world, attempt);

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
