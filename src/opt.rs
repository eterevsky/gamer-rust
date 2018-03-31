fn norm(v: &[f32]) -> f64 {
  v.iter().map(|&x| x as f64 * x as f64).sum()
}

pub fn minimize(
  _f: &Fn(&[f32]) -> f32,
  g: &Fn(&[f32]) -> Vec<f32>,
  params: &mut [f32],
) {
  let mut optimizer = AdamOptimizer::new(params.len(), 0.1);
  let mut i = 0;
  loop {
    i += 1;
    let grad = g(params);
    if norm(&grad) < 1E-14 * (1. + norm(params)) {
      break;
    };
    optimizer.gradient_step(params, grad.as_slice());
    if i > 200000 { break; }
  }
}

pub trait Optimizer {
  fn gradient_step(&mut self, param: &mut [f32], gradient: &[f32]);
}

pub struct AdamOptimizer {
  alpha: f32,
  beta1: f32,
  beta2: f32,
  eps: f32,
  m: Vec<f32>,
  v: Vec<f32>,
  t: i32,
}

impl AdamOptimizer {
  pub fn new(size: usize, alpha: f32) -> AdamOptimizer {
    AdamOptimizer {
      alpha: alpha,
      beta1: 0.9,
      beta2: 0.999,
      eps: 1E-8,
      m: vec![0.0; size],
      v: vec![0.0; size],
      t: 1,
    }
  }
}

impl Optimizer for AdamOptimizer {
  fn gradient_step(&mut self, param: &mut [f32], gradient: &[f32]) {
    assert_eq!(param.len(), gradient.len());
    self.t = self.t.saturating_add(1);

    for i in 0..param.len() {
      self.m[i] = self.beta1 * self.m[i] + (1.0 - self.beta1) * gradient[i];
      self.v[i] =
        self.beta2 * self.v[i] + (1.0 - self.beta2) * gradient[i] * gradient[i];
      let mbiased = self.m[i] / (1.0 - self.beta1.powi(self.t));
      let vbiased = self.v[i] / (1.0 - self.beta2.powi(self.t));
      param[i] -= self.alpha * mbiased / (vbiased.sqrt() + self.eps);
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
    let mut params = vec![0.0];
    minimize(&f, &g, params.as_mut_slice());
    assert_relative_eq!(-1.5, params[0], max_relative = 1E-4);
  }

  #[test]
  fn quadratic2d() {
    let f = |x: &[f32]| {
      3. * x[0] * x[0] - x[0] * x[1] + 2. * x[1] * x[1] + 2. * x[0] - x[1] + 1.
    };
    let g = |x: &[f32]| vec![6. * x[0] - x[1] + 2., -x[0] + 4. * x[1] - 1.];
    let mut params = vec![0.0; 2];
    minimize(&f, &g, params.as_mut_slice());
    assert_relative_eq!(-7. / 23., params[0], max_relative = 1E-4);
    assert_relative_eq!(4. / 23., params[1], max_relative = 1E-4);
  }
}
