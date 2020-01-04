use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use toml;

use taxi::world::Costs;

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum SolverChoice {
    Random,
    QLearner,
    RMax,
    FactoredRMax,
    MaxQ,
    DoorMax,
}

impl fmt::Display for SolverChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SolverChoice::Random => write!(f, "Random"),
            SolverChoice::QLearner => write!(f, "Q-Learner"),
            SolverChoice::RMax => write!(f, "RMax"),
            SolverChoice::FactoredRMax => write!(f, "FactoredRMax"),
            SolverChoice::MaxQ => write!(f, "MaxQ"),
            SolverChoice::DoorMax => write!(f, "DoorMax"),
        }
    }
}

pub trait ReportConfig {
    fn solver_choice(&self) -> SolverChoice;

    fn report(&self) -> bool {
        false
    }
}

#[derive(Deserialize, Debug)]
pub struct RandomSolverConfig {}

impl ReportConfig for RandomSolverConfig {
    fn solver_choice(&self) -> SolverChoice {
        SolverChoice::Random
    }
}

#[derive(Deserialize, Debug)]
pub struct QLearnerConfig {
    pub alpha: f64,
    pub gamma: f64,
    pub epsilon: f64,

    pub report: bool,
}

impl ReportConfig for QLearnerConfig {
    fn solver_choice(&self) -> SolverChoice {
        SolverChoice::QLearner
    }

    fn report(&self) -> bool {
        self.report
    }
}

#[derive(Deserialize, Debug)]
pub struct RMaxConfig {
    pub gamma: f64,
    pub known_count: f64,
    pub error_delta: f64,

    pub report: bool,
}

impl ReportConfig for RMaxConfig {
    fn solver_choice(&self) -> SolverChoice {
        SolverChoice::RMax
    }

    fn report(&self) -> bool {
        self.report
    }
}

#[derive(Deserialize, Debug)]
pub struct FactoredRMaxConfig {
    pub gamma: f64,
    pub known_count: f64,
    pub error_delta: f64,

    pub report: bool,
}

impl ReportConfig for FactoredRMaxConfig {
    fn solver_choice(&self) -> SolverChoice {
        SolverChoice::FactoredRMax
    }

    fn report(&self) -> bool {
        self.report
    }
}

#[derive(Deserialize, Debug)]
pub struct MaxQConfig {
    pub alpha: f64,
    pub gamma: f64,
    pub epsilon: f64,
    pub report: bool,
    pub show_learning: bool,
}

impl ReportConfig for MaxQConfig {
    fn solver_choice(&self) -> SolverChoice {
        SolverChoice::MaxQ
    }

    fn report(&self) -> bool {
        self.report || self.show_learning
    }
}

#[derive(Deserialize, Debug)]
pub struct DoorMaxConfig {
    pub gamma: f64,
    pub use_reward_learner: bool,
    pub known_count: f64,
    pub error_delta: f64,

    pub report: bool,
}

impl ReportConfig for DoorMaxConfig {
    fn solver_choice(&self) -> SolverChoice {
        SolverChoice::DoorMax
    }

    fn report(&self) -> bool {
        self.report
    }
}

#[derive(Deserialize, Debug)]
pub struct Probe {
    pub taxi_pos: (i32, i32),
    pub passenger_loc: Option<char>,
    pub destination_loc: char,
    pub max_steps: usize,
}

#[derive(Deserialize, Debug)]
pub struct Replay {
    pub solver: SolverChoice,
    pub taxi_pos: (i32, i32),
    pub passenger_loc: Option<char>,
    pub destination_loc: char,
    pub max_steps: usize,
}

#[derive(Deserialize, Debug)]
pub struct CostsConfig {
    pub movement: f64,
    pub miss_pickup: f64,
    pub miss_dropoff: f64,
    pub empty_dropoff: f64,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Configuration {
    pub world: String,
    pub costs: CostsConfig,
    // TOML only allows for signed 64 bit integers, not unsigned.
    pub root_seed: Option<(i64, i64)>,
    pub rerun_seeds: Vec<(i64, i64)>,
    pub probes: Vec<Probe>,
    pub max_trials: usize,
    pub max_trial_steps: usize,
    pub sessions: usize,
    pub random_solver: Option<RandomSolverConfig>,
    pub q_learner: Option<QLearnerConfig>,
    pub r_max: Option<RMaxConfig>,
    pub factored_r_max: Option<FactoredRMaxConfig>,
    pub max_q: Option<MaxQConfig>,
    pub door_max: Option<DoorMaxConfig>,
    pub replay: Option<Replay>,
}

impl Configuration {
    pub fn from_file(filename: &str) -> Result<Configuration, Error> {
        let mut config_file = File::open(filename).map_err(|_| Error::OpenFailure {
            filename: String::from(filename),
        })?;

        let mut config_string = String::new();
        config_file
            .read_to_string(&mut config_string)
            .map_err(|_| Error::ReadFailure {
                filename: String::from(filename),
            })?;

        toml::from_str(&config_string).map_err(|error| Error::ParseFailure {
            filename: String::from(filename),
            error,
        })
    }
}

pub enum Error {
    OpenFailure {
        filename: String,
    },
    ReadFailure {
        filename: String,
    },
    ParseFailure {
        filename: String,
        error: toml::de::Error,
    },
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::OpenFailure { ref filename } => {
                write!(f, "Configuration - Failed to open file '{}'", filename)
            }
            Error::ReadFailure { ref filename } => {
                write!(f, "Configuration - Failed to read file '{}'", filename)
            }
            Error::ParseFailure {
                ref filename,
                ref error,
            } => write!(
                f,
                "Configuration - Failed to parse config file '{}' - {}",
                filename, error
            ),
        }
    }
}

impl Default for Configuration {
    fn default() -> Configuration {
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

        let costs = CostsConfig {
            movement: Costs::default().movement,
            miss_pickup: Costs::default().miss_pickup,
            miss_dropoff: Costs::default().miss_dropoff,
            empty_dropoff: Costs::default().empty_dropoff,
        };

        Configuration {
            world: String::from(world_str),
            costs,
            root_seed: None,
            rerun_seeds: Vec::new(),
            probes: Vec::new(),
            max_trials: 1,
            max_trial_steps: 100,
            sessions: 0,
            random_solver: None,
            q_learner: None,
            r_max: None,
            factored_r_max: None,
            max_q: None,
            door_max: None,
            replay: None,
        }
    }
}
