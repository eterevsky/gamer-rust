use std;
use std::f32;
use rand;
use rand::Rng;

use def::{AgentReport, Game, Evaluator, FeatureExtractor, State};
use minimax::minimax_fixed_depth;
use spec::{EvaluatorSpec, RegressionSpec};

pub trait Regression : std::fmt::Debug {
  type Hyperparameters;

  fn new(params: Vec<f32>, hyperparams: Self::Hyperparameters) -> Self;
  fn evaluate(&self, features: &Vec<f32>) -> f32;
  fn train1(&mut self, features: &Vec<f32>, expected: f32);
  fn init<S: State, E: FeatureExtractor<S>>(&mut self, extractor: &E);
  fn spec(&self) -> RegressionSpec;
}

#[derive(Debug)]
pub struct LinearRegression {
  pub b: Vec<f32>,
  speed: f32,
  regularization: f32
}

impl Regression for LinearRegression {
  type Hyperparameters = (f32, f32);

  fn new(params: Vec<f32>, hyperparams: (f32, f32)) -> LinearRegression {
    LinearRegression {
      b: params,
      speed: hyperparams.0,
      regularization: hyperparams.1
    }
  }

  fn evaluate(&self, features: &Vec<f32>) -> f32 {
    assert_eq!(features.len(), self.b.len());
    let linear_combination: f32 =
        self.b.iter().zip(features.iter()).map(|(x, y)| x * y).sum();
    linear_combination.tanh()
  }

  fn train1(&mut self, features: &Vec<f32>, expected: f32) {
    let linear_combination: f32 =
        self.b.iter().zip(features.iter()).map(|(x, y)| x * y).sum();
    let prediction = linear_combination.tanh();
    let activation_derivative = 1.0 - prediction.powi(2);
    let error = prediction - expected;
    let feature_coef = self.speed * 2.0 * error * activation_derivative;
    let regularization_coef = self.speed * 2.0 * self.regularization;
    for i in 0..self.b.len() {
      self.b[i] -= feature_coef * features[i] + regularization_coef * self.b[i];
    }
  }

  fn init<S: State, E: FeatureExtractor<S>>(&mut self, extractor: &E) {
    if self.b.is_empty() {
      self.b = vec![0.0; extractor.nfeatures()];
    } else {
      assert_eq!(self.b.len(), extractor.nfeatures());
    }
  }

  fn spec(&self) -> RegressionSpec {
    RegressionSpec {
      speed: self.speed,
      regularization: self.regularization,
      b: self.b.clone()
    }
  }
}

pub struct FeatureEvaluator<G, E, R>
where G: Game, E: FeatureExtractor<G::State>, R: Regression {
  game: &'static G,
  extractor: E,
  regression: R,
  training_minimax_depth: u32
}

impl<G, E, R> FeatureEvaluator<G, E, R>
where G: Game, E: FeatureExtractor<G::State>, R: Regression {
  pub fn new(game: &'static G, extractor: E, regression: R, training_minimax_depth: u32) -> Self {
    FeatureEvaluator {
      game,
      extractor,
      regression,
      training_minimax_depth
    }
  }
}

impl<G, E, R> Evaluator<G::State> for FeatureEvaluator<G, E, R>
where G: Game, E: FeatureExtractor<G::State>, R: Regression {
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

  fn train(&mut self, steps: u64) {
    let discount = 0.999;
    let random_prob = 0.1;
    let callback = &|&_, _| ();
    let mut state = self.game.new_game();
    let mut rng = rand::weak_rng();

    for step in 0..steps {
      if state.is_terminal() {
        state = self.game.new_game();
      }

      let report = minimax_fixed_depth(&state, self, self.training_minimax_depth, discount);
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

  fn spec(&self) -> EvaluatorSpec {
    EvaluatorSpec::Features {
      extractor: self.extractor.spec(),
      regression: self.regression.spec(),
      training_minimax_depth: self.training_minimax_depth
    }
  }
}

#[cfg(test)]
mod test {

use super::*;
use subtractor::{Subtractor, SubtractorFeatureExtractor};

#[test]
fn train_linear_regression_subtractor() {
  let game = Subtractor::default(21, 4);
  let extractor = SubtractorFeatureExtractor::new(10);
  let regression = LinearRegression::new(vec![0.0; 10], (0.1, 0.001));
  let mut evaluator = FeatureEvaluator::new(game, extractor, regression, 1);
  evaluator.train(5000);

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