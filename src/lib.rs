#[macro_use]
extern crate lazy_static;
extern crate rand;

mod board;
pub mod def;
pub mod feature_evaluator;
pub mod gomoku;
pub mod hexapawn;
pub mod minimax;
pub mod random_agent;
mod status;
pub mod subtractor;
pub mod terminal_evaluator;