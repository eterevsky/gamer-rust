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
mod feature_evaluator;
#[macro_use]
pub mod games;
pub mod gomoku;
mod human_agent;
pub mod ladder;
pub mod minimax;
mod random_agent;
pub mod registry;
pub mod spec;
mod status;
pub mod subtractor;
mod terminal_evaluator;