use rand;
use rand::Rng;
use spec::EvaluatorSpec;
use std::time::{Duration, Instant};

use agents::MinimaxAgent;
use def::{Agent, Evaluator, FeatureExtractor, Game, Regression, State, Trainer};
use super::FeatureEvaluator;


pub struct AnnealingTrainer<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State>,
  R: Regression,
{
  game: &'static G,
  extractor: E,
  regression: R,
  step_size: f32,
  minimax_depth: u32,
  temperature: f32,
  steps: u64,
}

impl<G, E, R> AnnealingTrainer<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State>,
  R: Regression,
{
  pub fn new(
    game: &'static G,
    extractor: E,
    regression: R,
    step_size: f32,
    minimax_depth: u32,
    temperature: f32,
  ) -> Self {
    AnnealingTrainer {
      game,
      extractor,
      regression,
      step_size,
      minimax_depth,
      temperature,
      steps: 0,
    }
  }
}

impl<G, E, R> Evaluator<G::State> for AnnealingTrainer<G, E, R>
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
    unreachable!()
  }
}

impl<G, E, R> Trainer<G> for AnnealingTrainer<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State> + Clone + 'static,
  R: Regression + 'static,
{
  fn train(&mut self, steps: u64, time_limit: Duration) {
    let mut rng = rand::weak_rng();
    let mut last_report = Instant::now();

    let deadline = if time_limit != Duration::new(0, 0) {
      Some(Instant::now() + time_limit)
    } else {
      None
    };

    let mut current_agent =
      MinimaxAgent::new_boxed(self.build_evaluator(), self.minimax_depth, None);

    for _step in 0..steps {
      if let Some(d) = deadline {
        if Instant::now() >= d {
          break;
        }
      }

      let mut new_regression = self.regression.clone();
      for param in new_regression.mut_params().iter_mut() {
        *param += (rng.gen_range(-1, 2) as i32 as f32) * self.step_size;
      }

      let new_agent = MinimaxAgent::new_boxed(
        Box::new(FeatureEvaluator::new(
          self.game,
          self.extractor.clone(),
          new_regression.clone(),
        )),
        self.minimax_depth,
        None,
      );

      let new_plays_first = rng.gen_weighted_bool(2);
      let mut state = self.game.new_game();
      while !state.is_terminal() {
        let m = if new_plays_first != state.get_player() {
          current_agent.select_move(&state).unwrap().get_move()
        } else {
          new_agent.select_move(&state).unwrap().get_move()
        };
        state.play(m).unwrap();
      }

      let new_won = new_plays_first != (state.get_payoff().unwrap() < 0.0);

      if new_won
        || self.temperature > 0.0
          && rng.next_f32() < (-(self.steps as f32) / self.temperature).exp()
      {
        current_agent = new_agent;
        self.regression = new_regression;
      }

      self.steps += 1;

      if Instant::now() - last_report > Duration::new(10, 0) {
        println!("Step {}, temperature {}", self.steps, if self.temperature == 0.0 { 0.0 } else { (-(self.steps as f32) / self.temperature).exp()});
        self.extractor.report(self.regression.spec());
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

  use super::*;
  use evaluators::LinearRegressionTanh;
  use games::{Subtractor, SubtractorFeatureExtractor};

  #[test]
  fn train_subtractor() {
    let game = Subtractor::default(21, 4);
    let extractor = SubtractorFeatureExtractor::new(5);
    let regression = LinearRegressionTanh::zeros(5, 0.001);

    let mut trainer = AnnealingTrainer::new(
      game,
      extractor,
      regression,
      0.001, // step size
      1,     // minimax depth
      0.0,
    );

    trainer.train(200, Duration::new(0, 0));
    let evaluator = trainer.build_evaluator();

    // let game4 = Subtractor::new(4, 4);
    // let game5 = Subtractor::new(5, 4);

    // assert!(
    //   evaluator.evaluate(&game4.new_game())
    //     < evaluator.evaluate(&game5.new_game())
    // );
  }

}
