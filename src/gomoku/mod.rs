pub use self::gomoku::Gomoku;
pub use self::gomoku::GomokuState;

mod gomoku;
mod gomoku_move;
mod util;

#[cfg(test)]
mod gomoku_test;
