mod human;
pub mod mcts;
pub mod minimax;
mod random;

pub use self::human::HumanAgent;
pub use self::minimax::{MinimaxAgent, minimax_fixed_depth};
pub use self::random::RandomAgent;