pub mod agents;
mod board;
pub mod def;
mod equal_policy;
pub mod evaluators;
#[macro_use]
pub mod games;
pub mod ladder;
mod opt;
mod ratings;
pub mod registry;
pub mod spec;
mod status;

pub use self::evaluators::train_subtractor_eval;
