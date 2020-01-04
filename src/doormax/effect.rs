use std;
use std::fmt;

use crate::state;
use crate::state::State;
use crate::world::World;

pub enum Error {
    InvalidState(state::Error),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::InvalidState(ref state_error) => {
                write!(f, "Failed to create state from effect: {:?}", state_error)
            }
        }
    }
}

impl From<state::Error> for Error {
    fn from(error: state::Error) -> Self {
        Error::InvalidState(error)
    }
}

pub trait Effect
where
    Self: std::marker::Sized,
{
    fn generate_effects(old_state: &State, new_state: &State) -> Option<Self>;

    fn apply(&self, world: &World, state: &State) -> Result<State, Error>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChangeTaxiX {
    delta: i32,
}

impl ChangeTaxiX {
    pub fn new(delta: i32) -> Self {
        ChangeTaxiX { delta }
    }
}

impl Effect for ChangeTaxiX {
    fn generate_effects(old_state: &State, new_state: &State) -> Option<Self> {
        let old_x = old_state.get_taxi().x;
        let new_x = new_state.get_taxi().x;

        if old_x != new_x {
            let delta = new_x - old_x;
            Some(ChangeTaxiX::new(delta))
        } else {
            None
        }
    }

    fn apply(&self, world: &World, state: &State) -> Result<State, Error> {
        let new_taxi_x = state.get_taxi().x + self.delta;

        Ok(State::build(
            world,
            (new_taxi_x, state.get_taxi().y),
            state.get_passenger(),
            state.get_destination(),
        )?)
    }
}

impl fmt::Display for ChangeTaxiX {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChangeTaxiX({})", self.delta)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChangeTaxiY {
    delta: i32,
}

impl ChangeTaxiY {
    pub fn new(delta: i32) -> Self {
        ChangeTaxiY { delta }
    }
}

impl Effect for ChangeTaxiY {
    fn generate_effects(old_state: &State, new_state: &State) -> Option<Self> {
        let old_y = old_state.get_taxi().y;
        let new_y = new_state.get_taxi().y;

        if old_y != new_y {
            let delta = new_y - old_y;
            Some(ChangeTaxiY::new(delta))
        } else {
            None
        }
    }

    fn apply(&self, world: &World, state: &State) -> Result<State, Error> {
        let new_taxi_y = state.get_taxi().y + self.delta;

        Ok(State::build(
            world,
            (state.get_taxi().x, new_taxi_y),
            state.get_passenger(),
            state.get_destination(),
        )?)
    }
}

impl fmt::Display for ChangeTaxiY {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChangeTaxiY({})", self.delta)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChangePassenger {
    on_destination: bool,
}

impl ChangePassenger {
    pub fn new(on_destination: bool) -> Self {
        ChangePassenger { on_destination }
    }
}

impl Effect for ChangePassenger {
    fn generate_effects(old_state: &State, new_state: &State) -> Option<Self> {
        let old_passenger = old_state.get_passenger();
        let new_passenger = new_state.get_passenger();

        if old_passenger != new_passenger {
            // Assume that a non-None value for new_passenger
            // means that the passenger was placed on the destination.
            Some(ChangePassenger::new(new_passenger.is_some()))
        } else {
            None
        }
    }

    fn apply(&self, world: &World, state: &State) -> Result<State, Error> {
        let passenger = if self.on_destination {
            Some(state.get_destination())
        } else {
            None
        };

        Ok(State::build(
            world,
            (state.get_taxi().x, state.get_taxi().y),
            passenger,
            state.get_destination(),
        )?)
    }
}

impl fmt::Display for ChangePassenger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChangePassenger({:#?})", self.on_destination)
    }
}
