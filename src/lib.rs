#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[macro_use]
extern crate enum_map;





pub mod actions;
pub mod distribution;
pub mod doormax;
pub mod factoredrmax;
pub mod maxq;
pub mod position;
pub mod qlearner;
pub mod random_solver;
pub mod rmax;
pub mod runner;
pub mod state;
pub mod state_indexer;
pub mod world;
