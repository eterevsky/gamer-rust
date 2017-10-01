use std;
use rand;
use rand::Rng;

use def::{Game, Evaluator, State};

pub trait FeatureExtractor<'g, S: State<'g>> {
  type FeatureVector;
  /// Returns a feature vector for a given position from the point of view of
  /// the acting player.
  fn extract(&self, state: &S) -> Self::FeatureVector;
}

pub trait Regression<FV> {
  type Parameters;
  type Hyperparameters;

  fn new(params: Self::Parameters, hyperparams: Self::Hyperparameters) -> Self;
  fn export(&self) -> Self::Parameters;
  fn evaluate(&self, features: &FV) -> f32;
  fn train1(&mut self, features: &FV, expected: f32);
}

#[derive(Debug)]
pub struct LinearRegression {
  b: Vec<f32>,
  speed: f32
}

impl Regression<Vec<f32>> for LinearRegression {
  type Parameters = Vec<f32>;
  type Hyperparameters = f32;

  fn new(params: Vec<f32>, hyperparams: f32) -> LinearRegression {
    LinearRegression {
      b: params,
      speed: hyperparams
    }
  }

  fn export(&self) -> Vec<f32> {
    self.b.clone()
  }

  fn evaluate(&self, features: &Vec<f32>) -> f32 {
    assert_eq!(features.len(), self.b.len());
    self.b.iter().zip(features.iter()).map(|(x, y)| x * y).sum()
  }

  fn train1(&mut self, features: &Vec<f32>, expected: f32) {
    let prediction = self.evaluate(features);
    let error = prediction - expected;
    for i in 0..self.b.len() {
      self.b[i] -= self.speed * 2.0 * error * features[i];
    }
  }
}

pub struct FeatureEvaluator<'g, FV, FE, G, R>
  where FE: FeatureExtractor<'g, G::State, FeatureVector=FV>,
        R: Regression<FV>,
        G: Game<'g> + 'g {
  game: &'g G,
  extractor: FE,
  pub regression: R
}

impl<'g, FV, FE, G, R> FeatureEvaluator<'g, FV, FE, G, R>
    where FE: FeatureExtractor<'g, G::State, FeatureVector=FV>,
        R: Regression<FV>,
        G: Game<'g> + 'g {
  pub fn new(game: &'g G, extractor: FE, regression: R) -> Self {
    FeatureEvaluator {
      game,
      extractor,
      regression
    }
  }

  pub fn train(&mut self, nmoves: u32, discount: f32, random_prob: f32) {
    let mut state = self.game.new_game();
    let mut rng = rand::weak_rng();

    for _ in 0..nmoves {
      if state.is_terminal() {
        state = self.game.new_game();
      }

      let mut best_move = None;
      let mut best_score = std::f32::MIN;

      for m in state.iter_moves() {
        let mut state_clone = state.clone();
        state_clone.play(m).unwrap();
        let score = if state_clone.is_terminal() {
          if state.get_player() {
            state_clone.get_payoff().unwrap()
          } else {
            -state_clone.get_payoff().unwrap()
          }
        } else {
          let features = self.extractor.extract(&state_clone);
          let next_score = self.regression.evaluate(&features);
          if state_clone.get_player() == state.get_player() {
            next_score
          } else {
            -next_score
          }
        };

        if score > best_score {
          best_move = Some(m);
          best_score = score;
        }
      }

      assert!(best_move.is_some());
      best_score *= discount;

      self.regression.train1(&self.extractor.extract(&state), best_score);

      if rng.next_f32() < random_prob {
        let m = state.get_random_move(&mut rng).unwrap();
        state.play(m).unwrap();
      } else {
        state.play(best_move.unwrap()).unwrap();
      }
    }
  }
}

impl<'g, FV, FE, G, R> Evaluator<'g, G::State>
    for FeatureEvaluator<'g, FV, FE, G, R>
    where FE: FeatureExtractor<'g, G::State, FeatureVector=FV>,
          R: Regression<FV>,
          G: Game<'g> {
  fn evaluate(&self, state: &G::State) -> f32 {
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

use std::iter;

use super::*;
use subtractor::{Subtractor, SubtractorFeatureExtractor};

#[test]
fn train_linear_regression_subtractor() {
  let game = Subtractor::new(21, 4);
  let extractor = SubtractorFeatureExtractor::new(10);
  let regression = LinearRegression::new(
      iter::repeat(0.0).take(10).collect(),
      0.01);
  let mut evaluator = FeatureEvaluator::new(&game, extractor, regression);
  evaluator.train(10000, 0.999, 0.1);

  for i in 0..12 {
    let game = Subtractor::new(i, 4);
    let score = evaluator.evaluate(&game.new_game());
    if i % 4 == 0 {
      assert!(-1.1 < score && score < -0.9, "score for {} is {}", i, score);
    } else {
      assert!(0.9 < score && score < 1.1, "score for {} is {}", i, score);
    }
  }
}

}