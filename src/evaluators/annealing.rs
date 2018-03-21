use rand;
use rand::{Rng, XorShiftRng};
use spec::EvaluatorSpec;
use std::cell::RefCell;
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
  ngames: u32,
  steps: u64,
  rng: RefCell<XorShiftRng>
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
    ngames: u32
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
      rng: RefCell::new(rand::weak_rng())
    }
  }

  fn current_temperature(&self) -> f32 {
    if self.temperature == 0.0 {
      0.0
    } else {
      (-(self.steps as f32) / self.temperature).exp()
    }
  }

  // Plays self.ngames games with alternating first player. Returns the sum
  // of payoffs for player1.
  fn run_games(&self, player1: &MinimaxAgent<G::State>, player2: &MinimaxAgent<G::State>) -> f32 {
    let mut payoff = 0.0;
    let mut player1_first = self.rng.borrow_mut().gen_weighted_bool(2);

    for _game in 0..self.ngames {
      let mut state = self.game.new_game();
      while !state.is_terminal() {
        let m = if player1_first == state.get_player() {
          player1.select_move(&state).unwrap().get_move()
        } else {
          player2.select_move(&state).unwrap().get_move()
        };
        state.play(m).unwrap();
      }
      payoff += if player1_first { state.get_payoff().unwrap() } else { -state.get_payoff().unwrap() };
      player1_first = !player1_first;
    }

    payoff
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
    unreachable!("AnnealingTrainer shouldn't be converted to EvaluatorSpec directly.")
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

      let new_payoff = self.run_games(&new_agent, &current_agent);

      if new_payoff > 0.0 || rng.next_f32() < self.current_temperature() {
        current_agent = new_agent;
        self.regression = new_regression;

        if Instant::now() - last_report > Duration::from_secs(5) {
          println!("Step {}, temperature {}", self.steps, self.current_temperature());
          self.extractor.report(&self.regression);
          last_report = Instant::now();
        }
      }

      self.steps += 1;
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
      5,     // minimax depth
      10.0,  // temperature
      3      // ngames
    );

    trainer.train(500, Duration::new(0, 0));
    let evaluator = trainer.build_evaluator();

    for n in 1..11 {
      println!("eval({}) = {}", n, evaluator.evaluate(&Subtractor::new(n, 4).new_game()))
    }

    let game4 = Subtractor::new(4, 4);
    let game5 = Subtractor::new(5, 4);

    assert!(
      evaluator.evaluate(&game4.new_game())
        < evaluator.evaluate(&game5.new_game())
    );
  }

}
