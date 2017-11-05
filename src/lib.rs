#[cfg(test)]
#[macro_use]
extern crate approx;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod board;
pub mod def;
pub mod feature_evaluator;
pub mod games;
mod human_agent;
pub mod ladder;
pub mod minimax;
mod opt;
mod random_agent;
mod ratings;
pub mod registry;
mod sampler_evaluator;
pub mod spec;
mod status;
pub mod terminal_evaluator;