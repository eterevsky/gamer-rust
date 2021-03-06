use rand::{Rng, FromEntropy};
use rand::rngs::SmallRng;
use std::time::{Duration, Instant};

use crate::agents::minimax_fixed_depth;
use crate::def::{AgentReport, Evaluator, FeatureExtractor, Game, Regression, State, Trainer};
use crate::opt::{AdamOptimizer, Optimizer};
use crate::spec::EvaluatorSpec;

use super::FeatureEvaluator;

pub struct ReinforceTrainer<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State>,
  R: Regression,
{
  game: &'static G,
  extractor: E,
  regression: R,
  minimax_depth: u32,
  random_prob: f32,
  optimizer: AdamOptimizer,
  steps: u64,
}

impl<G, E, R> ReinforceTrainer<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State>,
  R: Regression,
{
  pub fn new(
    game: &'static G,
    extractor: E,
    regression: R,
    minimax_depth: u32,
    random_prob: f32,
    alpha: f32,
  ) -> Self {
    let len = regression.params().len();
    ReinforceTrainer {
      game,
      extractor,
      regression,
      minimax_depth,
      random_prob,
      optimizer: AdamOptimizer::new(len, alpha),
      steps: 0,
    }
  }
}

impl<G, E, R> Evaluator<G::State> for ReinforceTrainer<G, E, R>
where G: Game, E: FeatureExtractor<G::State>, R: Regression {
  fn evaluate(&self, state: &G::State) -> f32 {
    if let Some(score) = state.payoff() {
      return score;
    }
    let features = self.extractor.extract(state);
    let player_score = self.regression.evaluate(&features);
    if state.player() {
      player_score
    } else {
      -player_score
    }
  }

  fn spec(&self) -> EvaluatorSpec {
    unreachable!()
  }
}

impl<G, E, R> Trainer<G> for ReinforceTrainer<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State> + Clone + 'static,
  R: Regression + 'static,
{
  fn train(&mut self, steps: u64, time_limit: Duration) {
    let discount = 0.999;
    let mut rng = SmallRng::from_entropy();
    let mut last_report = Instant::now();

    let deadline = if time_limit != Duration::new(0, 0) {
      Some(Instant::now() + time_limit)
    } else {
      None
    };

    for _step in 0..steps {
      if let Some(d) = deadline {
        if Instant::now() >= d {
          break;
        }
      }

      let mut state = self.game.new_game();

      while !state.is_terminal() {
        let report = minimax_fixed_depth(&state, self, self.minimax_depth, discount);
        let score = if state.player() { report.score } else { -report.score };
        let gradient = self.regression.gradient1(
            &self.extractor.extract(&state), score);
        self.optimizer.gradient_step(self.regression.mut_params(), gradient.as_slice());
        self.steps += 1;

        if rng.gen_bool(self.random_prob as f64) {
          let m = state.get_random_move(&mut rng).unwrap();
          state.play(m).unwrap();
        } else {
          state.play(report.get_move()).unwrap();
        }
      }

      if Instant::now() - last_report > Duration::new(10, 0) {
        self.extractor.report(&self.regression);
        last_report = Instant::now();
      }
    }
  }

  fn build_evaluator(&self) -> Box<Evaluator<G::State>> {
    Box::new(FeatureEvaluator::new(
      self.game,
      self.extractor.clone(),
      self.regression.clone(),
    ))
  }
}

#[cfg(test)]
mod test {

use crate::evaluators::LinearRegressionTanh;
use crate::games::{Subtractor, SubtractorFeatureExtractor};

use super::*;

#[test]
fn train_subtractor() {
  let game = Subtractor::default(21, 4);
  let extractor = SubtractorFeatureExtractor::new(5);
  let regression = LinearRegressionTanh::zeros(5, 0.001);

  let mut trainer = ReinforceTrainer::new(
    game,
    extractor,
    regression,
    5,           // minimax_depth
    0.1,         // random_prob
    0.1);        // AdamOptimizer alpha

  trainer.train(200, Duration::new(0, 0));
  let evaluator = trainer.build_evaluator();

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