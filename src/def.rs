//! General definitions of games, moves, players etc.

use std::fmt;

pub trait Game {
  fn nplayers(&self) -> u32;
}

pub trait GameState : Game + Clone + fmt::Display {
  type Move: Copy + Clone;
  type Player: Copy;

  fn play(&mut self, Self::Move) -> Result<(), &'static str>;
  fn get_player(&self) -> Self::Player;
  fn get_payoff(&self, Self::Player) -> Option<f32>;
  fn is_terminal(&self) -> bool;
}

pub trait MoveGenerator<S: GameState> {
  fn generate(&self, state: &S) -> Vec<S::Move>;
}

pub trait MoveSelector<S: GameState> {
  /// None if the state is terminal.
  fn select(&self, state: &S) -> Option<S::Move>;
}

pub trait Evaluator<S: GameState> {
  fn evaluate(&self, state: &S) -> f32;

  fn evaluate_move(&self, state: &S, m: S::Move) -> f32 {
    let mut state_clone = state.clone();
    state_clone.play(m).ok();
    self.evaluate(&state_clone)
  }
}

pub trait TerminalEvaluator<S: GameState> {
  // Some(..) -- if terminal, should be compatible with payoff
  // None -- if not terminal
  fn evaluate_terminal(&self, state: &S) -> Option<f32>;
}
