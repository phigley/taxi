#[cfg(test)]
#[macro_use]
extern crate assert_matches;

extern crate rand;
extern crate float_cmp;

pub mod position;
pub mod world;
pub mod state;
pub mod actions;
pub mod distribution;
pub mod state_indexer;
pub mod runner;
pub mod random_solver;
pub mod qlearner;
pub mod rmax;
pub mod factoredrmax;
