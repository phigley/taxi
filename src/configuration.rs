
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use toml;

#[derive(Deserialize, Debug)]
pub enum ReplayMode {
    None,
    First,
    FirstSuccess,
    FirstFailure,
}

#[derive(Deserialize, Debug)]
pub struct InitialState {
    pub taxi_pos: (i32, i32),
    pub passenger_loc: char,
    pub destination_loc: char,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Configuration {
    pub world: String,
    pub initial_states: Vec<InitialState>,
    pub max_steps: u32,
    pub trials: u32,
    pub replay_mode: ReplayMode,
}

impl Configuration {
    pub fn from_file(filename: &str) -> Result<Configuration, Error> {
        let mut config_file = File::open(filename).or(Err(Error::OpenFailure {
            filename: String::from(filename),
        }))?;

        let mut config_string = String::new();
        config_file.read_to_string(&mut config_string).or(Err(
            Error::ReadFailure { filename: String::from(filename) },
        ))?;

        toml::from_str(&config_string).map_err(|error| {
            Error::ParseFailure {
                filename: String::from(filename),
                error,
            }
        })
    }
}

pub enum Error {
    OpenFailure { filename: String },
    ReadFailure { filename: String },
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
            } => {
                write!(
                    f,
                    "Configuration - Failed to parse config file '{}' - {}",
                    filename,
                    error
                )
            }
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
            initial_states: vec![],
            trials: 1,
            max_steps: 500,
            replay_mode: ReplayMode::First,
        }
    }
}
