use serde::ser::{Serialize, Deserialize};
use std::marker::PhantomData;

use def;

trait Regression<FV> {
  type Parameters: Serialize, Deserialize;
  type Hyperparameters: Serialize, Deserialize;

  fn new(params: Parameters, hyperparams: Hyperparameters) -> Self;
  fn export(&self) -> Parameters;
  fn evaluate(&self, features: &FV) -> f32;
  fn train1(&mut self, features: &FV, expected: f32);
}

struct LinearRegression {
  b: Vec<f32>
}

impl Regression<Vec<f32>> for LinearRegression {
  type Parameters = Vec<f32>;
  type HyperParameters = ();

  fn new(params: Vec<f32>, hyperparams: ()) -> LinearRegression {
    LinearRegression {
      b: params
    }
  }

  fn export(&self) -> Vec<f32> {
    self.b.clone()
  }

  fn evaluate(&self, features: &Vec<f32>) -> f32 {
    assert_eq!(feature.len(), self.b.len());
    b.iter().zip(features.iter()).map(|(x, y)| x * y).sum()
  }

  fn train1(&mut self, features: &Vec<f32>, expected: f32) {
    // Not implemented
  }
}

struct FeatureEvaluator<'g, FV, FE, R, S>
  where FE: def::FeatureExtractor<FeatureVector=FV>,
        R: Regression<FV>,
        S: def::State<'g> {
  extractor: FE,
  regression: R,
  _s: PhantomData<S>,
  _l: PhantomData<&'g ()>
}

impl<'g, FV, FE, R, S> def::Evaluator<'g, S> for FeatureEvaluator<'g, FV, FE, R, S>
    where FE: def::FeatureExtractor<'g, S>,
          R: Regression<Parameters=FV>,
          S: def::State<'g> {
  
}