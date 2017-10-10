use std;
use std::f32;
use std::fmt;
use rand;
use rand::Rng;

use def::{AgentReport, Game, Evaluator, State};
use minimax::minimax_fixed_depth;

pub trait FeatureExtractor<'g, S: State<'g>> {
  type FeatureVector;
  /// Returns a feature vector for a given position from the point of view of
  /// the acting player.
  fn extract(&self, state: &S) -> Self::FeatureVector;
}

impl<'e, 'g, S: State<'g>, E> FeatureExtractor<'g, S> for &'e E
    where E: FeatureExtractor<'g, S> {
  type FeatureVector = E::FeatureVector;
  fn extract(&self, state: &S) -> E::FeatureVector {
    (*self).extract(state)
  }
}

pub trait Regression<FV> : std::fmt::Debug {
  type Parameters;
  type Hyperparameters;

  fn new(params: Self::Parameters, hyperparams: Self::Hyperparameters) -> Self;
  fn export(&self) -> Self::Parameters;
  fn evaluate(&self, features: &FV) -> f32;
  fn train1(&mut self, features: &FV, expected: f32);
}

#[derive(Debug)]
pub struct LinearRegression {
  pub b: Vec<f32>,
  speed: f32,
  regularization: f32
}

impl Regression<Vec<f32>> for LinearRegression {
  type Parameters = Vec<f32>;
  type Hyperparameters = (f32, f32);

  fn new(params: Vec<f32>, hyperparams: (f32, f32)) -> LinearRegression {
    LinearRegression {
      b: params,
      speed: hyperparams.0,
      regularization: hyperparams.1
    }
  }

  fn export(&self) -> Vec<f32> {
    self.b.clone()
  }

  fn evaluate(&self, features: &Vec<f32>) -> f32 {
    assert_eq!(features.len(), self.b.len());
    let linear_combination: f32 = self.b.iter().zip(features.iter()).map(|(x, y)| x * y).sum();
    linear_combination.tanh()
  }

  fn train1(&mut self, features: &Vec<f32>, expected: f32) {
    let linear_combination: f32 = self.b.iter().zip(features.iter()).map(|(x, y)| x * y).sum();
    let prediction = linear_combination.tanh();
    let activation_derivative = 1.0 - prediction.powi(2);
    let error = prediction - expected;
    let feature_coef = self.speed * 2.0 * error * activation_derivative;
    let regularization_coef = self.speed * 2.0 * self.regularization;
    for i in 0..self.b.len() {
      self.b[i] -= feature_coef * features[i] + regularization_coef * self.b[i];
    }
  }
}

pub struct FeatureEvaluator<'g, FV, FE, G, R>
  where FE: FeatureExtractor<'g, G::State, FeatureVector=FV>,
        R: Regression<FV>,
        G: Game<'g> + 'g {
  game: &'g G,
  extractor: FE,
  pub regression: R,
}

impl<'g, FV, FE, G, R> FeatureEvaluator<'g, FV, FE, G, R>
    where FE: FeatureExtractor<'g, G::State, FeatureVector=FV>,
        R: Regression<FV> + fmt::Debug,
        G: Game<'g> + 'g {
  pub fn new(game: &'g G, extractor: FE, regression: R) -> Self {
    FeatureEvaluator {
      game,
      extractor,
      regression
    }
  }

  pub fn train(&mut self, nmoves: u32, discount: f32, random_prob: f32, callback: &Fn(&Self, u32)) {
    let mut state = self.game.new_game();
    let mut rng = rand::weak_rng();

    for step in 0..nmoves {
      if state.is_terminal() {
        state = self.game.new_game();
      }

      let report = minimax_fixed_depth(&state, self, 2, discount);
      let score = if state.get_player() { report.score } else { -report.score };
      self.regression.train1(&self.extractor.extract(&state), score);

      if rng.next_f32() < random_prob {
        let m = state.get_random_move(&mut rng).unwrap();
        state.play(m).unwrap();
      } else {
        state.play(report.get_move()).unwrap();
      }

      callback(self, step);
    }
  }
}

impl<'g, FV, FE, G, R> Evaluator<'g, G::State>
    for FeatureEvaluator<'g, FV, FE, G, R>
    where FE: FeatureExtractor<'g, G::State, FeatureVector=FV>,
          R: Regression<FV>,
          G: Game<'g> {
  fn evaluate(&self, state: &G::State) -> f32 {
    if let Some(score) = state.get_payoff() {
      return score;
    }
    let features = self.extractor.extract(state);
    let player_score = self.regression.evaluate(&features);
    if state.get_player() {
      player_score
    } else {
      -player_score
    }
  }
}

#[cfg(test)]
mod test {

use super::*;
use subtractor::{Subtractor, SubtractorFeatureExtractor};

#[test]
fn train_linear_regression_subtractor() {
  let game = Subtractor::new(21, 4);
  let extractor = SubtractorFeatureExtractor::new(10);
  let regression = LinearRegression::new(vec![0.0; 10], (0.1, 0.001));
  let mut evaluator = FeatureEvaluator::new(&game, extractor, regression);
  evaluator.train(1000, 0.999, 0.01, &|_, _| ());

  for i in 0..12 {
    let game = Subtractor::new(i, 4);
    let score = evaluator.evaluate(&game.new_game());
    if i % 4 == 0 {
      assert!(-1.0 <= score && score < -0.5, "score for {} is {}", i, score);
    } else {
      assert!(0.5 < score && score <= 1.0, "score for {} is {}", i, score);
    }
  }
}

}