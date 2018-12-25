//! General traits for games, players and related concepts.

use rand;
use std::fmt;
use std::ops::Deref;
use std::time::Duration;

use crate::spec::{AgentSpec, EvaluatorSpec, FeatureExtractorSpec, RegressionSpec};

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
  fn player(&self) -> bool;

  /// Returns true if position is terminal.
  fn is_terminal(&self) -> bool {
    self.payoff() == None
  }

  /// Returns payoff for the first player if position is terminal, None
  /// otherwize.
  fn payoff(&self) -> Option<f32>;

  /// Returns an iterator over all legal moves in the given position.
  fn iter_moves<'s>(&'s self) -> Box<Iterator<Item = Self::Move> + 's>;

  /// Generates a random valid move or None if the position is terminal.
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
  /// Returns an agent report that includes the selected move.
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

  fn evaluate_for_player(&self, state: &S, player: bool) -> f32 {
    if player {
      self.evaluate(state)
    } else {
      -self.evaluate(state)
    }
  }

  fn spec(&self) -> EvaluatorSpec;

  fn report(&self) {}
}

impl<'a, S: State, U: ?Sized + Evaluator<S>> Evaluator<S> for &'a U {
  fn evaluate(&self, state: &S) -> f32 {
    (*self).evaluate(state)
  }

  fn spec(&self) -> EvaluatorSpec {
    (*self).spec()
  }

  fn report(&self) {
    (*self).report()
  }
}

impl<S: State, U: ?Sized + Evaluator<S>> Evaluator<S> for Box<U> {
  fn evaluate(&self, state: &S) -> f32 {
    self.deref().evaluate(state)
  }

  fn spec(&self) -> EvaluatorSpec {
    self.deref().spec()
  }

  fn report(&self) {
    self.deref().report()
  }  
}

/// A policy trait that generates for a state the set of moves with probabilities.
pub trait Policy<S: State> {
  /// Generate moves for a given state with their probabilities. Generates a
  /// subset of all valid moves in the given state. The probabilities for
  /// different moves should sum up to 1.
  fn get_moves(&self, state: &S) -> Vec<(S::Move, f32)>;
}

impl<'a, S: State, P: Policy<S>> Policy<S> for &'a P {
  fn get_moves(&self, state: &S) -> Vec<(S::Move, f32)> {
    (*self).get_moves(state)
  }
}

pub trait FeatureExtractor<S: State> {
  fn nfeatures(&self) -> usize;

  /// Returns a feature vector for a given position from the point of view of
  /// the acting player.
  fn extract(&self, state: &S) -> Vec<f32>;

  fn spec(&self) -> FeatureExtractorSpec;

  fn report<R: Regression>(&self, _regression: &R) {}
}

pub trait Regression: Clone {
  fn params<'a>(&'a self) -> &'a [f32];
  fn mut_params<'a>(&'a mut self) -> &'a mut [f32];
  fn evaluate(&self, features: &[f32]) -> f32;
  fn gradient1(&self, features: &[f32], value: f32) -> Vec<f32>;
  fn gradient(&self, feature_sets: &[Vec<f32>], values: &[f32]) -> Vec<f32> {
    let mut grad = vec![0.0; feature_sets[0].len()];
    for (ref features, &v) in feature_sets.iter().zip(values.iter()) {
      let part_grad = self.gradient1(&features[..], v);
      for i in 0..grad.len() {
        grad[i] += part_grad[i];
      }
    }
    grad
  }
  fn spec(&self) -> RegressionSpec;
}

pub trait Trainer<G: Game> {
  fn train(&mut self, steps: u64, time_limit: Duration);
  fn build_evaluator(&self) -> Box<Evaluator<G::State>>;
}
