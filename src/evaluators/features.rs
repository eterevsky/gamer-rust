use std::f32;

use def::{Evaluator, FeatureExtractor, Game, Regression, State};
use spec::EvaluatorSpec;

pub struct FeatureEvaluator<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State>,
  R: Regression,
{
  _game: &'static G,
  extractor: E,
  regression: R,
}

impl<G, E, R> FeatureEvaluator<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State>,
  R: Regression,
{
  pub fn new(game: &'static G, extractor: E, regression: R) -> Self {
    FeatureEvaluator {
      _game: game,
      extractor,
      regression,
    }
  }
}

impl<G, E, R> Evaluator<G::State> for FeatureEvaluator<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State>,
  R: Regression,
{
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

  fn spec(&self) -> EvaluatorSpec {
    EvaluatorSpec::Features {
      extractor: self.extractor.spec(),
      regression: self.regression.spec(),
    }
  }
}

#[cfg(test)]
mod test {

  use super::*;
  use games::{Subtractor, SubtractorFeatureExtractor};
  use evaluators::LinearRegressionTanh;

  #[test]
  fn evaluate_subtractor() {
    let game = Subtractor::default(21, 4);
    let extractor = SubtractorFeatureExtractor::new(5);
    let regression =
      LinearRegressionTanh::new(&[5.0, 0.0, 0.0, -10.0, 0.0], 0.001);
    let evaluator = FeatureEvaluator::new(game, extractor, regression);

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