#[cfg(test)]
#[macro_use]
extern crate assert_matches;

extern crate float_cmp;
extern crate rand;

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
pub mod maxq;
pub mod doormax;
