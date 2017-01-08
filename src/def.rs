//! General definitions of games, moves, players etc.

use rand;
use std::fmt;

pub trait Game {
  type State: GameState;

  fn new() -> Self;
  fn new_game(&self) -> Self::State;
}

pub trait GameState : Clone + fmt::Display {
  type Move: Copy + Clone;
  type Player: Copy;

  fn play(&mut self, Self::Move) -> Result<(), &'static str>;
  fn play_random_move<R: rand::Rng>(&mut self, rng: &mut R) -> Result<(), &'static str>;
  fn get_player(&self) -> Self::Player;
  fn get_payoff_for_player1(&self) -> Option<f32>;
  fn get_payoff(&self, Self::Player) -> Option<f32>;
  fn is_terminal(&self) -> bool;
}

// pub trait MoveGenerator<'a, S: GameState<'a>> {
//   fn generate(&self, state: &S) -> Vec<S::Move>;
// }
//
// pub trait MoveSelector<'a, S: GameState<'a>> {
//   /// None if the state is terminal.
//   fn select(&self, state: &S) -> Option<S::Move>;
// }
//
// pub trait Evaluator<'a, S: GameState<'a>> {
//   fn evaluate(&self, state: &S) -> f32;
//
//   fn evaluate_move(&self, state: &S, m: S::Move) -> f32 {
//     let mut state_clone = state.clone();
//     state_clone.play(m).ok();
//     self.evaluate(&state_clone)
//   }
// }
//
// pub trait TerminalEvaluator<'a, S: GameState<'a>> {
//   // Some(..) -- if terminal, should be compatible with payoff
//   // None -- if not terminal
//   fn evaluate_terminal(&self, state: &S) -> Option<f32>;
// }
