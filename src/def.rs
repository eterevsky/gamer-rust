//! General definitions of games, moves, players etc.

use rand;
use std::fmt;

pub trait Game {
  type State: State;

  fn new() -> Self;
  fn new_game(&self) -> Self::State;
}

pub trait State : Clone + fmt::Display {
  type Move: Copy + Clone;
  type Player: Copy;

  fn play(&mut self, Self::Move) -> Result<(), &'static str>;
  fn get_random_move<R: rand::Rng>(&self, rng: &mut R) -> Option<Self::Move>;
  fn play_random_move<R: rand::Rng>(&mut self, rng: &mut R) -> Result<(), &'static str>;
  fn get_player(&self) -> Self::Player;
  fn get_payoff_for_player1(&self) -> Option<f32>;
  fn get_payoff(&self, Self::Player) -> Option<f32>;
  fn is_terminal(&self) -> bool;
}

pub trait MoveGenerator<S: State> {
  fn generate(&self, state: &S) -> Vec<S::Move>;
}

pub trait Agent<S: State> {
  // None if the state is terminal.
  fn select_move(&mut self, state: &S) -> Option<S::Move>;
}

pub struct RandomAgent<R: rand::Rng + Clone> {
  rng: R
}

impl<R: rand::Rng + Clone> RandomAgent<R> {
  pub fn new(rng: R) -> Self {
    RandomAgent {
      rng: rng.clone()
    }
  }
}

impl<S: State, R: rand::Rng + Clone> Agent<S> for RandomAgent<R> {
  fn select_move(&mut self, state: &S) -> Option<S::Move> {
    state.get_random_move(&mut self.rng)
  }
}

pub trait Evaluator<S: State> {
  fn evaluate(&self, state: &S, player: S::Player) -> f32;

  fn evaluate_move(&self, state: &S, m: S::Move, player: S::Player) -> f32 {
    let mut state_clone = state.clone();
    state_clone.play(m).ok();
    self.evaluate(&state_clone, player)
  }
}
