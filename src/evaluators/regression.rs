use crate::def::Regression;
use crate::spec::RegressionSpec;

#[derive(Clone, Debug)]
pub struct LinearRegressionTanh {
  params: Vec<f32>,
  regularization: f32
}

impl LinearRegressionTanh {
  pub fn zeros(input_size: usize, regularization: f32) -> LinearRegressionTanh {
    LinearRegressionTanh {
      params: vec![0.0; input_size],
      regularization
    }
  }

  pub fn new(params: &[f32], regularization: f32) -> LinearRegressionTanh {
    LinearRegressionTanh {
      params: params.to_vec(),
      regularization
    }
  }

}

impl Regression for LinearRegressionTanh {
  fn mut_params<'a>(&'a mut self) -> &'a mut [f32] {
    self.params.as_mut_slice()
  }

  fn params<'a>(&'a self) -> &'a [f32] {
    self.params.as_slice()
  }

  fn evaluate(&self, features: &[f32]) -> f32 {
    assert_eq!(features.len(), self.params.len());
    let linear_combination: f32 =
        self.params.iter().zip(features.iter()).map(|(x, y)| x * y).sum();
    linear_combination.tanh()
  }

  fn gradient1(&self, features: &[f32], value: f32) -> Vec<f32> {
    let linear_combination: f32 =
        self.params.iter().zip(features.iter()).map(|(x, y)| x * y).sum();
    let prediction = linear_combination.tanh();
    let activation_derivative = 1.0 - prediction.powi(2);
    let error = prediction - value;
    let feature_coef = 2.0 * error * activation_derivative;
    let regularization_coef = 2.0 * self.regularization;

    let mut grad = Vec::with_capacity(features.len());
    for (p, f) in self.params.iter().zip(features) {
      grad.push(feature_coef * f + regularization_coef * p);
    }
    grad
  }

  fn spec(&self) -> RegressionSpec {
    RegressionSpec {
      params: self.params.clone(),
      regularization: self.regularization
    }
  }
}

