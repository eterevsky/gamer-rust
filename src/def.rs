//! General definitions of games, moves, players etc.

use rand;
use std::fmt;

pub trait Game<'a> {
  type State: State<'a>;

  fn new() -> Self;
  fn new_game(&'a self) -> Self::State;
}

pub trait State<'a> : Clone + fmt::Display {
  type Move: Copy + Clone;

  fn play(&mut self, Self::Move) -> Result<(), &'static str>;
  fn iter_moves<'b>(&'b self) -> Box<Iterator<Item=Self::Move> + 'b>;
  fn get_random_move<R: rand::Rng>(&self, rng: &mut R) -> Option<Self::Move>;
  fn play_random_move<R: rand::Rng>(&mut self, rng: &mut R) -> Result<(), &'static str>;
  fn get_player(&self) -> bool;
  /// Payoff for player #0.
  fn get_payoff(&self) -> Option<f32>;
  fn is_terminal(&self) -> bool;
}

pub trait Agent<'a, S: State<'a> + 'a> {
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

impl<'a, S: State<'a> + 'a, R: rand::Rng + Clone> Agent<'a, S> for RandomAgent<R> {
  fn select_move(&mut self, state: &S) -> Option<S::Move> {
    state.get_random_move(&mut self.rng)
  }
}

pub trait Evaluator<'a, S: State<'a> + 'a> {
  fn evaluate(&self, state: &S) -> f32;
}
