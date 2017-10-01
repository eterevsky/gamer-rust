//! General traits for games, players and related concepts.

use rand;
use std::fmt;

/// A trait for a game rules set.
pub trait Game<'g> {
  type State: State<'g>;
  /// Starts a new game and return the game state before the first move of
  /// the first player.
  fn new_game(&'g self) -> Self::State;
}

/// A trait for a game state. Lifetime parameter `'g` corresponds to Game object
/// lifetime.
pub trait State<'g>: Clone + fmt::Display {
  type Move: Copy + Clone + fmt::Display;

  /// Returns true if it's the turn of the first player.
  fn get_player(&self) -> bool;

  /// Returns true if position is terminal.
  fn is_terminal(&self) -> bool {
    self.get_payoff() == None
  }

  /// Returns payoff for the first player if position is terminal, None
  /// otherwize.
  fn get_payoff(&self) -> Option<f32>;

  /// Returns an iterator over all legal moves in the given position.
  fn iter_moves<'s>(&'s self) -> Box<Iterator<Item = Self::Move> + 's>;

  /// Returns a random valid move or None if the position is terminal.
  fn get_random_move<R: rand::Rng>(&self, rng: &mut R) -> Option<Self::Move>;

  /// Plays a move.
  fn play(&mut self, m: Self::Move) -> Result<(), &'static str>;
}

pub trait AgentReport<M: fmt::Display>: fmt::Display {
  fn get_move(&self) -> M;
}

pub trait Agent<'g, S: State<'g>> {
  type Report: AgentReport<S::Move>;

  /// Returns a pair of a move and agent report if
  fn select_move(
    &mut self,
    state: &S,
  ) -> Result<Self::Report, &'static str>;
}

pub trait Evaluator<'g, S: State<'g>> {
  /// Evaluates the state and returns the score for the first player, regardless
  /// whose turn it is.
  fn evaluate(&self, state: &S) -> f32;
}