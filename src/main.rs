#[macro_use]
extern crate serde_derive;

mod configuration;
mod replay;

use rand::Rng;
use std::env;
use std::fmt;
use std::time;

use rand_pcg::Pcg64Mcg;

use rayon::prelude::*;

use crate::configuration::{Configuration, ReportConfig, SolverChoice};

use taxi::distribution::MeasureDistribution;
use taxi::state::State;
use taxi::world::{Costs, World};

use taxi::doormax::DoorMax;
use taxi::factoredrmax::FactoredRMax;
use taxi::maxq::MaxQ;
use taxi::qlearner::QLearner;
use taxi::random_solver::RandomSolver;
use taxi::rmax::RMax;
use taxi::runner::{run_training_session, Probe, Runner};

#[cfg(not(windows))]
use std::io;

#[cfg(not(windows))]
use termion::event;

#[cfg(not(windows))]
use termion::input::TermRead;

#[cfg(not(windows))]
use crate::replay::Replay;

enum AppError {
    NoConfiguration,
    Configuration(configuration::Error),
    World(taxi::world::Error),
    BuildProbes(taxi::state::Error),
    Runner(taxi::runner::Error),
    #[cfg(not(windows))]
    ReplayRunnerNotConfigured(SolverChoice),
    #[cfg(not(windows))]
    ReplayTraining(taxi::runner::Error),
    #[cfg(not(windows))]
    ReplayState(taxi::state::Error),
    #[cfg(not(windows))]
    Replay(io::Error),
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AppError::NoConfiguration => write!(f, "Configuration file not specified."),
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
            #[cfg(not(windows))]
            AppError::ReplayRunnerNotConfigured(ref runner_type) => write!(
                f,
                "Attempting to replay {:?} solver with out a valid configuration \
                 for that solver.",
                runner_type
            ),
            #[cfg(not(windows))]
            AppError::ReplayTraining(ref runner_error) => {
                write!(f, "Failed to run training for replay:\n{:?}", runner_error)
            }
            #[cfg(not(windows))]
            AppError::ReplayState(ref state_error) => {
                write!(f, "Failed to build replay state:\n{:?}", state_error)
            }
            #[cfg(not(windows))]
            AppError::Replay(ref replay_error) => {
                write!(f, "Failed to replay:\n{:?}", replay_error)
            }
        }
    }
}

fn main() -> Result<(), AppError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(AppError::NoConfiguration);
    }

    let config = Configuration::from_file(&args[1]).map_err(AppError::Configuration)?;

    let costs = Costs::new(
        config.costs.movement,
        config.costs.miss_pickup,
        config.costs.miss_dropoff,
        config.costs.empty_dropoff,
    );
    let world = World::build_from_str(&config.world, costs).map_err(AppError::World)?;
    let probes = build_probes(&config, &world)?;

    let root_seed = if let Some((seed_high, seed_low)) = config.root_seed {
        (seed_high as u128).rotate_left(64) + (seed_low as u128)
    } else {
        rand::random()
    };

    if config.sessions > 0 {
        let mut results = Vec::new();

        if let Some(ref random_config) = config.random_solver {
            gather_stats(
                RandomSolver::new,
                random_config,
                &world,
                &probes,
                &config,
                root_seed,
                &mut results,
            )?;
        };

        if let Some(ref qlearner_config) = config.q_learner {
            gather_stats(
                || {
                    QLearner::new(
                        &world,
                        qlearner_config.alpha,
                        qlearner_config.gamma,
                        qlearner_config.epsilon,
                    )
                },
                qlearner_config,
                &world,
                &probes,
                &config,
                root_seed,
                &mut results,
            )?;
        };

        if let Some(ref rmax_config) = config.r_max {
            gather_stats(
                || {
                    RMax::new(
                        &world,
                        rmax_config.gamma,
                        rmax_config.known_count,
                        rmax_config.error_delta,
                    )
                },
                rmax_config,
                &world,
                &probes,
                &config,
                root_seed,
                &mut results,
            )?;
        };

        if let Some(ref factored_rmax_config) = config.factored_r_max {
            gather_stats(
                || {
                    FactoredRMax::new(
                        &world,
                        factored_rmax_config.gamma,
                        factored_rmax_config.known_count,
                        factored_rmax_config.error_delta,
                    )
                },
                factored_rmax_config,
                &world,
                &probes,
                &config,
                root_seed,
                &mut results,
            )?;
        };

        if let Some(ref maxq_config) = config.max_q {
            gather_stats(
                || {
                    MaxQ::new(
                        &world,
                        maxq_config.alpha,
                        maxq_config.gamma,
                        maxq_config.epsilon,
                        maxq_config.show_learning,
                    )
                },
                maxq_config,
                &world,
                &probes,
                &config,
                root_seed,
                &mut results,
            )?;
        };

        if let Some(ref doormax_config) = config.door_max {
            gather_stats(
                || {
                    DoorMax::new(
                        &world,
                        doormax_config.gamma,
                        doormax_config.use_reward_learner,
                        doormax_config.known_count,
                        doormax_config.error_delta,
                    )
                },
                doormax_config,
                &world,
                &probes,
                &config,
                root_seed,
                &mut results,
            )?;
        };

        println!();

        for (solver_choice, stats) in results {
            let (avg_steps, stddev_steps) = stats.distribution.get_distribution();

            let elapsed_time =
                stats.duration.as_secs() as f64 + f64::from(stats.duration.subsec_nanos()) * 1e-9;

            println!(
                "{:?} - finished {} sessions in {:.1} average steps with stddev of {:.2} \
                 in {:.3} secs. Using seed [{}, {}]",
                solver_choice,
                stats.distribution.get_count() as usize,
                avg_steps,
                stddev_steps,
                elapsed_time,
                root_seed.rotate_right(64) as i64,
                root_seed as i64,
            );
        }
    }

    for (seed_high, seed_low) in config.rerun_seeds {
        let seed = (seed_high as u128).rotate_left(64) + (seed_low as u128);

        if let Some(ref random_config) = config.random_solver {
            rerun_session(
                RandomSolver::new,
                random_config,
                &world,
                &probes,
                (config.max_trials, config.max_trial_steps),
                seed,
            )?;
        };

        if let Some(ref qlearner_config) = config.q_learner {
            rerun_session(
                || {
                    QLearner::new(
                        &world,
                        qlearner_config.alpha,
                        qlearner_config.gamma,
                        qlearner_config.epsilon,
                    )
                },
                qlearner_config,
                &world,
                &probes,
                (config.max_trials, config.max_trial_steps),
                seed,
            )?;
        };

        if let Some(ref rmax_config) = config.r_max {
            rerun_session(
                || {
                    RMax::new(
                        &world,
                        rmax_config.gamma,
                        rmax_config.known_count,
                        rmax_config.error_delta,
                    )
                },
                rmax_config,
                &world,
                &probes,
                (config.max_trials, config.max_trial_steps),
                seed,
            )?;
        };

        if let Some(ref factored_rmax_config) = config.factored_r_max {
            rerun_session(
                || {
                    FactoredRMax::new(
                        &world,
                        factored_rmax_config.gamma,
                        factored_rmax_config.known_count,
                        factored_rmax_config.error_delta,
                    )
                },
                factored_rmax_config,
                &world,
                &probes,
                (config.max_trials, config.max_trial_steps),
                seed,
            )?;
        };

        if let Some(ref maxq_config) = config.max_q {
            rerun_session(
                || {
                    MaxQ::new(
                        &world,
                        maxq_config.alpha,
                        maxq_config.gamma,
                        maxq_config.epsilon,
                        maxq_config.show_learning,
                    )
                },
                maxq_config,
                &world,
                &probes,
                (config.max_trials, config.max_trial_steps),
                seed,
            )?;
        };

        if let Some(ref doormax_config) = config.door_max {
            rerun_session(
                || {
                    DoorMax::new(
                        &world,
                        doormax_config.gamma,
                        doormax_config.use_reward_learner,
                        doormax_config.known_count,
                        doormax_config.error_delta,
                    )
                },
                doormax_config,
                &world,
                &probes,
                (config.max_trials, config.max_trial_steps),
                seed,
            )?;
        };
    }

    #[cfg(not(windows))]
    {
        let mut rng = rand::thread_rng();

        if let Some(ref replay_config) = config.replay {
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
                    if let Some(ref qlearner_config) = config.q_learner {
                        run_replay(
                            &mut QLearner::new(
                                &world,
                                qlearner_config.alpha,
                                qlearner_config.gamma,
                                qlearner_config.epsilon,
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
                    if let Some(ref rmax_config) = config.r_max {
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
                    if let Some(ref factored_rmax_config) = config.factored_r_max {
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
                SolverChoice::MaxQ => {
                    if let Some(ref maxq_config) = config.max_q {
                        run_replay(
                            &mut MaxQ::new(
                                &world,
                                maxq_config.alpha,
                                maxq_config.gamma,
                                maxq_config.epsilon,
                                maxq_config.show_learning,
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
                SolverChoice::DoorMax => {
                    if let Some(ref doormax_config) = config.door_max {
                        run_replay(
                            &mut DoorMax::new(
                                &world,
                                doormax_config.gamma,
                                doormax_config.use_reward_learner,
                                doormax_config.known_count,
                                doormax_config.error_delta,
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
        )
        .map_err(AppError::BuildProbes)?;

        probes.push(Probe::new(state, probe_config.max_steps));
    }

    Ok(probes)
}

#[derive(Default)]
struct Stats {
    distribution: MeasureDistribution,
    duration: time::Duration,
}

fn gather_stats<B, Rnr>(
    builder: B,
    report_config: &dyn ReportConfig,
    world: &World,
    probes: &[Probe],
    config: &Configuration,
    root_seed: u128,
    results: &mut Vec<(SolverChoice, Stats)>,
) -> Result<(), AppError>
where
    B: Fn() -> Rnr + Sync,
    Rnr: Runner + Sync,
{
    let mut seed_generator = Pcg64Mcg::new(root_seed);
    let session_ids: Vec<(usize, u128)> = (0..config.sessions)
        .map(|session_id| (session_id, seed_generator.gen()))
        .collect();

    let solver_choice = report_config.solver_choice();
    let report = report_config.report();

    let stats = session_ids
        .par_iter()
        .fold(
            || Ok(Stats::default()),
            |current_result, (session_number, seed)| -> Result<Stats, AppError> {
                current_result.and_then(|mut stats| {
                    let start_time = time::Instant::now();

                    let mut solver = builder();

                    let mut rng = Pcg64Mcg::new(*seed);

                    let training_step_count = run_training_session(
                        world,
                        probes,
                        config.max_trials,
                        config.max_trial_steps,
                        &mut solver,
                        &mut rng,
                    )
                    .map_err(AppError::Runner)?;

                    let duration = start_time.elapsed();
                    let elapsed_time =
                        duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) * 1e-9;

                    match training_step_count {
                        Some(num_steps) => {
                            println!(
                                "{:?} - Finished session {} [{}, {}] in {} steps in {:.3} secs.",
                                solver_choice,
                                session_number,
                                seed.rotate_right(64) as i64,
                                *seed as i64,
                                num_steps,
                                elapsed_time,
                            );
                            stats.distribution.add_value(num_steps as f64);
                        }
                        None => {
                            println!(
                                "{:?} - Failed session {} [{},{}] with maximums {} trials of {} steps \
                                 in {:.3} secs.",
                                solver_choice,
                                session_number,
                                seed.rotate_right(64) as i64,
                                *seed as i64,
                                config.max_trials,
                                config.max_trial_steps,
                                elapsed_time,
                            );
                        }
                    };

                    stats.duration += duration;

                    // This may overlap with other reports, should we guard with a mutex?
                    if report {
                        // println!("Report Reults #{} : BEGIN", session_number);
                        solver.report_training_result(world, training_step_count);
                        // println!("Report Reults #{} : END", session_number);
                    }

                    Ok(stats)
                })
            },
        )
        .reduce(
            || Ok(Stats::default()),
            |result_a: Result<Stats, AppError>,
             result_b: Result<Stats, AppError>|
             -> Result<Stats, AppError> {
                result_a.and_then(|mut stats_a| {
                    result_b.and_then(|stats_b| {
                        stats_a.distribution.add_distribution(&stats_b.distribution);
                        stats_a.duration += stats_b.duration;
                        Ok(stats_a)
                    })
                })
            },
        )?;

    results.push((solver_choice, stats));

    Ok(())
}

fn rerun_session<B, Rnr>(
    builder: B,
    report_config: &dyn ReportConfig,
    world: &World,
    probes: &[Probe],
    (max_trials, max_trial_steps): (usize, usize),
    seed: u128,
) -> Result<(), AppError>
where
    B: Fn() -> Rnr,
    Rnr: Runner,
{
    let solver_choice = report_config.solver_choice();

    let start_time = time::Instant::now();

    let mut solver = builder();
    let mut rng = Pcg64Mcg::new(seed);

    let training_step_count = run_training_session(
        world,
        probes,
        max_trials,
        max_trial_steps,
        &mut solver,
        &mut rng,
    )
    .map_err(AppError::Runner)?;

    let duration = start_time.elapsed();
    let elapsed_time = duration.as_secs() as f64 + f64::from(duration.subsec_nanos()) * 1e-9;

    match training_step_count {
        Some(num_steps) => {
            println!(
                "{:?} - Finished seed [{}, {}] in {} steps in {:.3} secs.",
                solver_choice,
                seed.rotate_right(64) as i64,
                seed as i64,
                num_steps,
                elapsed_time,
            );
        }
        None => {
            println!(
                "{:?} - Failed seed [{},{}] with maximums {} trials of {} steps \
                 in {:.3} secs.",
                solver_choice,
                seed.rotate_right(64) as i64,
                seed as i64,
                max_trials,
                max_trial_steps,
                elapsed_time,
            );
        }
    };

    solver.report_training_result(world, training_step_count);

    Ok(())
}

#[cfg(not(windows))]
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
        )
        .map_err(AppError::ReplayState)?;

        let attempt = solver.attempt(world, replay_state, replay_config.max_steps, &mut rng);

        let replay = Replay::new(world, attempt);
        replay.run().map_err(AppError::Replay)?;
    }

    Ok(())
}

#[cfg(not(windows))]
fn wait_for_input() -> Option<()> {
    println!("Press Enter to see replay.  q to exit.");

    loop {
        for c in io::stdin().keys() {
            match c {
                Ok(evt) => match evt {
                    event::Key::Char('q') | event::Key::Char('Q') => return None,
                    event::Key::Char('\n') => return Some(()),
                    _ => (),
                },
                Err(_) => return None,
            }
        }
    }
}
