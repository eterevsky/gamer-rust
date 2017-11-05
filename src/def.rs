//! General traits for games, players and related concepts.

use rand;
use std::fmt;
use std::time::Duration;

use spec::{AgentSpec, EvaluatorSpec, FeatureExtractorSpec, RegressionSpec};

/// A trait for a game rules set.
pub trait Game: 'static + Sync {
  type State: State;

  /// Starts a new game and return the game state before the first move of
  /// the first player.
  fn new_game(&self) -> Self::State;
}

/// A trait for a game state. Lifetime parameter `'g` corresponds to Game object
/// lifetime.
pub trait State: Clone + fmt::Display {
  type Move: 'static + Clone + Copy + fmt::Debug + fmt::Display;

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

  /// Undo a given move. The caller must make sure that the passed move was
  /// actually the last move to be played in the position.
  fn undo(&mut self, m: Self::Move) -> Result<(), &'static str>;

  /// Parse move, represented as a string.
  fn parse_move(&self, move_str: &str) -> Result<Self::Move, &'static str>;
}

pub trait AgentReport<M>: fmt::Display {
  fn get_move(&self) -> M;
}

pub trait Agent<S: State> {
  /// Returns a pair of a move and agent report if
  fn select_move(
    &self,
    state: &S,
  ) -> Result<Box<AgentReport<S::Move>>, &'static str>;

  fn spec(&self) -> AgentSpec;
}

pub trait Evaluator<S: State> {
  /// Evaluates the state and returns the score for the first player, regardless
  /// whose turn it is.
  fn evaluate(&self, state: &S) -> f32;

  /// For training evaluators -- train for a number of steps. For non-training
  /// evaluators -- keep non-implemented.
  fn train(&mut self, _steps: u64, _time_limit: Duration) {
    unreachable!()
  }

  fn spec(&self) -> EvaluatorSpec;
}

pub trait FeatureExtractor<S: State> {
  fn nfeatures(&self) -> usize;

  /// Returns a feature vector for a given position from the point of view of
  /// the acting player.
  fn extract(&self, state: &S) -> Vec<f32>;

  fn spec(&self) -> FeatureExtractorSpec;

  fn report(&self, _regression_spec: RegressionSpec) {}
}
