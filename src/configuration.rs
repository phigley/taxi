use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use toml;

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum SolverChoice {
    Random,
    QLearner,
    RMax,
    FactoredRMax,
    MaxQ,
}

impl fmt::Display for SolverChoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SolverChoice::Random => write!(f, "Random"),
            SolverChoice::QLearner => write!(f, "Q-Learner"),
            SolverChoice::RMax => write!(f, "RMax"),
            SolverChoice::FactoredRMax => write!(f, "FactoredRMax"),
            SolverChoice::MaxQ => write!(f, "MaxQ"),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct RandomSolverConfig {}

#[derive(Deserialize, Debug)]
pub struct QLearnerConfig {
    pub alpha: f64,
    pub gamma: f64,
    pub epsilon: f64,

    pub show_table: bool,
}

#[derive(Deserialize, Debug)]
pub struct RMaxConfig {
    pub gamma: f64,
    pub known_count: f64,
    pub error_delta: f64,
}

#[derive(Deserialize, Debug)]
pub struct FactoredRMaxConfig {
    pub gamma: f64,
    pub known_count: f64,
    pub error_delta: f64,
}

#[derive(Deserialize, Debug)]
pub struct MaxQConfig {
    pub alpha: f64,
    pub gamma: f64,
    pub epsilon: f64,
    pub show_table: bool,
    pub show_learning: bool,
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
#[serde(default)]
pub struct Configuration {
    pub world: String,
    pub probes: Vec<Probe>,
    pub max_trials: usize,
    pub max_trial_steps: usize,
    pub sessions: usize,
    pub random_solver: Option<RandomSolverConfig>,
    pub q_learner: Option<QLearnerConfig>,
    pub r_max: Option<RMaxConfig>,
    pub factored_r_max: Option<FactoredRMaxConfig>,
    pub max_q: Option<MaxQConfig>,
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

        Configuration {
            world: String::from(world_str),
            probes: vec![],
            max_trials: 1,
            max_trial_steps: 100,
            sessions: 1,
            random_solver: None,
            q_learner: None,
            r_max: None,
            factored_r_max: None,
            max_q: None,
            replay: None,
        }
    }
}
