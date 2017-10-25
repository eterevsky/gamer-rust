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
pub mod gomoku;
pub mod hexapawn;
mod human_agent;
pub mod ladder;
pub mod minimax;
pub mod play;
pub mod random_agent;
mod registry;
pub mod spec;
mod status;
pub mod subtractor;
pub mod terminal_evaluator;