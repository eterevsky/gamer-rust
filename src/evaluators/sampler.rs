use rand::{weak_rng, XorShiftRng};
use std::cell::RefCell;

use def::{Evaluator, State};
use spec::EvaluatorSpec;

#[derive(Clone, Debug)]
pub struct SamplerEvaluator {
  rng: RefCell<XorShiftRng>,
  nsamples: usize,
  discount: f64,
}

impl SamplerEvaluator {
  pub fn new(nsamples: usize, discount: f32) -> Self {
    SamplerEvaluator {
      rng: RefCell::new(weak_rng()),
      nsamples,
      discount: discount as f64,
    }
  }
}

impl<S: State> Evaluator<S> for SamplerEvaluator {
  fn evaluate(&self, state: &S) -> f32 {
    if state.is_terminal() {
      return state.payoff().unwrap();
    }
    let rng = &mut *self.rng.borrow_mut();
    let mut total_payoff: f64 = 0.0;
    for _ in 0..self.nsamples {
      let mut playout_state = state.clone();
      let mut moves = 0;
      while let Some(m) = playout_state.get_random_move(rng) {
        playout_state.play(m).unwrap();
        moves += 1;
      }
      let payoff = playout_state.payoff().unwrap() as f64;
      total_payoff += payoff * self.discount.powi(moves);
    }

    (total_payoff / self.nsamples as f64) as f32
  }

  fn spec(&self) -> EvaluatorSpec {
    EvaluatorSpec::Sampler {
      samples: self.nsamples,
      discount: self.discount as f32,
    }
  }
}

#[cfg(test)]
mod test {

use def::{Game, State};
use games::Subtractor;
use super::*;

#[test]
fn subtractor() {
  let game = Subtractor::new(1, 4);
  let mut state = game.new_game();
  let evaluator = SamplerEvaluator::new(1, 0.999);

  assert_eq!(0.999, evaluator.evaluate(&state));
  state.play(1).unwrap();
  assert_eq!(1.0, evaluator.evaluate(&state));
}

#[test]
fn subtractor_many_samples() {
  let game = Subtractor::new(3, 4);
  let state = game.new_game();
  let evaluator = SamplerEvaluator::new(1000, 0.75);

  let expected_evaluation =
      (1. / 3.) * 0.75 * 1. +                // play 3
      (1. / 3.) * 0.75 * 0.75 * (-1.) +      // play 2 1
      (1. / 6.) * 0.75 * 0.75 * (-1.) +      // play 1 2
      (1. / 6.) * 0.75 * 0.75 * 0.75 * 1.;   // play 1 1 1

  let actual_evaluation = evaluator.evaluate(&state);

  assert!((expected_evaluation - actual_evaluation).abs() < 0.1,
          "expected: {} actual: {}", expected_evaluation, actual_evaluation);
}

}