//! Generate new evaluators, evaluate them using a ladder.

use rand;
use rand::Rng;
use std::time::{Duration, Instant};

use def::{Evaluator, Game, Trainer};
use ladder::Ladder;
use registry::create_evaluator;
use spec::{AgentSpec, EvaluatorSpec, FeatureExtractorSpec, RegressionSpec};

pub struct LadderAnnealingTrainer<G: Game> {
  game: &'static G,
  extractor_spec: FeatureExtractorSpec,
  step_size: f32,
  minimax_depth: u32,
  temperature: f32,
  ngames: usize,
  steps: u64,
  ladder: Ladder,
  best_id: usize,
  best_regression_spec: RegressionSpec,
}

impl<G: Game> LadderAnnealingTrainer<G> {
  pub fn new(
    game: &'static G,
    extractor_spec: FeatureExtractorSpec,
    regression_spec: RegressionSpec,
    step_size: f32,
    minimax_depth: u32,
    temperature: f32,
    ngames: usize,
  ) -> Self {
    let mut ladder = Ladder::new(game, 8);
    ladder.add_participant(&AgentSpec::Random);
    LadderAnnealingTrainer {
      game,
      extractor_spec,
      step_size,
      minimax_depth,
      temperature,
      ngames,
      steps: 0,
      ladder,
      best_id: 0,
      best_regression_spec: regression_spec,
    }
  }

  fn current_temperature(&self) -> f32 {
    if self.temperature == 0.0 {
      0.0
    } else {
      0.01_f32.max((-(self.steps as f32) / self.temperature).exp())
    }
  }

  fn report(&self) {
    println!(
      "Step {}, temperature {}, rating {}",
      self.steps,
      self.current_temperature(),
      self.ladder.get_rating(self.best_id)
    );
    let agent_spec = self.ladder.get_participant(self.best_id);
    if let &AgentSpec::Minimax {
      depth: _,
      time_per_move: _,
      name: _,
      ref evaluator
    } = agent_spec
    {
      create_evaluator(self.game, evaluator).report();
    }
  }
}

impl<G: Game> Trainer<G> for LadderAnnealingTrainer<G> {
  fn train(&mut self, steps: u64, time_limit: Duration) {
    println!("steps: {}", steps);
    let mut rng = rand::weak_rng();
    let mut last_report = Instant::now();

    let deadline = if time_limit != Duration::new(0, 0) {
      Some(Instant::now() + time_limit)
    } else {
      None
    };

    for step in 1..steps {
      self.steps += 1;
      if let Some(d) = deadline {
        if Instant::now() >= d {
          break;
        }
      }

      let mut new_regression = self.best_regression_spec.clone();
      for param in new_regression.params.iter_mut() {
        *param += (rng.gen_range(-1, 2) as i32 as f32) * self.step_size;
      }

      let new_agent_spec = AgentSpec::Minimax {
        depth: self.minimax_depth,
        time_per_move: 0.0,
        name: format!("Annealing{}", step),
        evaluator: EvaluatorSpec::Features {
          extractor: self.extractor_spec.clone(),
          regression: new_regression.clone(),
        },
      };

      let (new_id, new_rating) = self
        .ladder
        .add_participant_and_rank(&new_agent_spec, self.ngames);

      if new_rating > self.ladder.get_rating(self.best_id)
        || rng.next_f32() < self.current_temperature()
      {
        self.best_id = new_id;
        self.best_regression_spec = new_regression;

        if Instant::now() - last_report > Duration::from_secs(5) {
          self.report();
          last_report = Instant::now();
        }
      }
    }

    self.ladder.print_all();
    println!("best_id: {}", self.best_id);
  }

  fn build_evaluator(&self) -> Box<Evaluator<G::State>> {
    let agent_spec = self.ladder.get_participant(self.best_id);

    if let &AgentSpec::Minimax {
      depth: _,
      time_per_move: _,
      name: _,
      ref evaluator,
    } = agent_spec
    {
      create_evaluator(self.game, evaluator)
    } else {
      panic!()
    }
  }
}
