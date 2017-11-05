use std::iter::repeat;

fn norm(v: &[f32]) -> f32 {
  v.iter().map(|x| x * x).sum()
}

pub fn minimize(
  f: &Fn(&[f32]) -> f32,
  g: &Fn(&[f32]) -> Vec<f32>,
  init: &[f32],
) -> Vec<f32> {
  let mut optimizer = AdamOptimizer::new(init);
  optimizer.set_step(0.1);
  for step in 0..100000000 {
    let grad = g(optimizer.params());
    if norm(&grad) < 1E-10 * (1. + norm(optimizer.params())) {
      break;
    };
    optimizer.gradient_step(grad.as_slice());
  }
  optimizer.params().to_vec()
}

trait Optimizer {
  fn gradient_step(&mut self, gradient: &[f32]);
  fn params<'a>(&'a self) -> &'a [f32];
}

struct AdamOptimizer {
  alpha: f32,
  beta1: f32,
  beta2: f32,
  eps: f32,
  params: Vec<f32>,
  m: Vec<f32>,
  v: Vec<f32>,
  t: i32,
  n: usize,
}

impl AdamOptimizer {
  fn new(init_params: &[f32]) -> AdamOptimizer {
    AdamOptimizer {
      alpha: 0.001,
      beta1: 0.9,
      beta2: 0.999,
      eps: 1E-8,
      params: init_params.to_vec(),
      m: repeat(0.0).take(init_params.len()).collect(),
      v: repeat(0.0).take(init_params.len()).collect(),
      t: 1,
      n: init_params.len(),
    }
  }

  fn set_step(&mut self, alpha: f32) {
    self.alpha = alpha;
  }

  fn report(&self, gradient: &[f32]) {
    println!(
      "t = {} params {:?} grad {:?}",
      self.t,
      self.params,
      gradient
    );
  }
}

impl Optimizer for AdamOptimizer {
  fn params<'a>(&'a self) -> &'a [f32] {
    self.params.as_slice()
  }

  fn gradient_step(&mut self, gradient: &[f32]) {
    self.t.saturating_add(1);

    for i in 0..self.n {
      self.m[i] = self.beta1 * self.m[i] + (1.0 - self.beta1) * gradient[i];
      self.v[i] =
        self.beta2 * self.v[i] + (1.0 - self.beta2) * gradient[i] * gradient[i];
      let mbiased = self.m[i] / (1.0 - self.beta1.powi(self.t));
      let vbiased = self.v[i] / (1.0 - self.beta2.powi(self.t));
      self.params[i] -= self.alpha * mbiased / (vbiased.sqrt() + self.eps);
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn quadratic1d() {
    let f = |x: &[f32]| x[0] * x[0] + 3. * x[0] + 1.;
    let g = |x: &[f32]| vec![2. * x[0] + 3.];
    let xmin = minimize(&f, &g, &[0.0]);
    assert_relative_eq!(-1.5, xmin[0], max_relative = 1E-4);
  }

  #[test]
  fn quadratic2d() {
    let f = |x: &[f32]| {
      3. * x[0] * x[0] - x[0] * x[1] + 2. * x[1] * x[1] + 2. * x[0] - x[1] + 1.
    };
    let g = |x: &[f32]| vec![6. * x[0] - x[1] + 2., -x[0] + 4. * x[1] - 1.];
    let xmin = minimize(&f, &g, &[0., 0.]);
    assert_relative_eq!(-7. / 23., xmin[0], max_relative = 1E-4);
    assert_relative_eq!(4. / 23., xmin[1], max_relative = 1E-4);
  }
}
