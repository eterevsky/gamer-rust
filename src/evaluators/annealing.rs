use rand::{Rng, FromEntropy};
use rand::rngs::SmallRng;
use std::cell::RefCell;
use std::time::{Duration, Instant};

use crate::agents::MinimaxAgent;
use crate::def::{Agent, Evaluator, FeatureExtractor, Game, Regression, State, Trainer};
use crate::spec::EvaluatorSpec;

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
  ngames: u32,
  steps: u64,
  rng: RefCell<SmallRng>,
}

impl<G, E, R> AnnealingTrainer<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State> + Clone,
  R: Regression,
{
  pub fn new(
    game: &'static G,
    extractor: E,
    regression: R,
    step_size: f32,
    minimax_depth: u32,
    temperature: f32,
    ngames: u32,
  ) -> Self {
    AnnealingTrainer {
      game,
      extractor,
      regression,
      step_size,
      minimax_depth,
      temperature,
      ngames,
      steps: 0,
      rng: RefCell::new(SmallRng::from_entropy()),
    }
  }

  fn current_temperature(&self) -> f32 {
    if self.temperature == 0.0 {
      0.0
    } else {
      0.01_f32.max( (-(self.steps as f32) / self.temperature).exp() )
    }
  }

  // Plays self.ngames games with alternating first player. Returns the sum
  // of payoffs for player1.
  fn run_games<A1: Agent<G::State>, A2: Agent<G::State>>(
      &self, player1: &A1, player2: &A2) -> f32 {
    let mut payoff = 0.0;
    let mut player1_first = self.rng.borrow_mut().gen_ratio(1, 2);

    for _game in 0..self.ngames {
      let mut state = self.game.new_game();
      while !state.is_terminal() {
        let m = if player1_first == state.player() {
          player1.select_move(&state).unwrap().get_move()
        } else {
          player2.select_move(&state).unwrap().get_move()
        };
        state.play(m).unwrap();
      }
      payoff += if player1_first {
        state.payoff().unwrap()
      } else {
        -state.payoff().unwrap()
      };
      player1_first = !player1_first;
    }

    payoff
  }

  fn build_unboxed_evaluator(&self) -> FeatureEvaluator<G, E, R> {
    FeatureEvaluator::new(
      self.game,
      self.extractor.clone(),
      self.regression.clone(),
    )
  }
}

impl<G, E, R> Evaluator<G::State> for AnnealingTrainer<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State>,
  R: Regression,
{
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
    unreachable!(
      "AnnealingTrainer shouldn't be converted to EvaluatorSpec directly."
    )
  }
}

impl<G, E, R> Trainer<G> for AnnealingTrainer<G, E, R>
where
  G: Game,
  E: FeatureExtractor<G::State> + Clone + 'static,
  R: Regression + 'static,
{
  fn train(&mut self, steps: u64, time_limit: Duration) {
    let mut rng = SmallRng::from_entropy();
    let mut last_report = Instant::now();

    let deadline = if time_limit != Duration::new(0, 0) {
      Some(Instant::now() + time_limit)
    } else {
      None
    };

    let mut current_agent =
      MinimaxAgent::new(self.build_unboxed_evaluator(), self.minimax_depth, None);

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

      let new_agent = MinimaxAgent::new(
        FeatureEvaluator::new(
          self.game,
          self.extractor.clone(),
          new_regression.clone(),
        ),
        self.minimax_depth,
        None,
      );

      let new_payoff = self.run_games(&new_agent, &current_agent);

      if new_payoff > 0.0 || rng.gen_bool(self.current_temperature() as f64) {
        current_agent = new_agent;
        self.regression = new_regression;

        if Instant::now() - last_report > Duration::from_secs(5) {
          println!(
            "Step {}, temperature {}",
            self.steps,
            self.current_temperature()
          );
          self.extractor.report(&self.regression);
          last_report = Instant::now();
        }
      }

      self.steps += 1;
    }
  }

  fn build_evaluator(&self) -> Box<Evaluator<G::State>> {
    Box::new(self.build_unboxed_evaluator())
  }
}

#[cfg(test)]
mod test {

  use super::*;
  use crate::evaluators::LinearRegressionTanh;
  use crate::evaluators::train_subtractor_eval::check_evaluator;
  use crate::games::{Subtractor, SubtractorFeatureExtractor};

  #[test]
  fn train_subtractor() {
    let game = Subtractor::default(21, 4);
    let extractor = SubtractorFeatureExtractor::new(10);
    let regression = LinearRegressionTanh::zeros(10, 0.001);

    let mut trainer = AnnealingTrainer::new(
      game,
      extractor,
      regression,
      0.01,  // step size
      2,     // minimax depth
      1.0,  // temperature
      1,     // ngames
    );

    trainer.train(10000, Duration::new(0, 0));
    let evaluator = trainer.build_evaluator();

    println!();
    for n in 1..22 {
      println!(
        "eval({}) = {}",
        n,
        evaluator.evaluate(&Subtractor::new(n, 4).new_game())
      )
    }

    assert_eq!(21, check_evaluator(evaluator.as_ref()));
  }

}
