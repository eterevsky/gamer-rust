use std::any::Any;
use std::mem::transmute;
use std::time::Duration;

use def::{Agent, Evaluator, FeatureExtractor, Game, Regression, State, Trainer};
use evaluators::{AnnealingTrainer, FeatureEvaluator, LinearRegressionTanh,
                 ReinforceTrainer, SamplerEvaluator, TerminalEvaluator};
use games::{Gomoku, GomokuLineFeatureExtractor, Hexapawn,
            HexapawnNumberOfPawnsExtractor, Subtractor,
            SubtractorFeatureExtractor};
use agents::{HumanAgent, MinimaxAgent, RandomAgent};
use spec::{AgentSpec, EvaluatorSpec, FeatureExtractorSpec, RegressionSpec,
           TrainerSpec, TrainingSpec};


pub fn create_agent<G: Game>(
  game: &'static G,
  spec: &AgentSpec,
) -> Box<Agent<G::State>> {
  match spec {
    &AgentSpec::Random => Box::new(RandomAgent::new()),

    &AgentSpec::Human => Box::new(HumanAgent {}),

    &AgentSpec::Minimax {
      depth,
      time_per_move,
      evaluator: ref evaluator_spec,
      name: _,
    } => {
      let evaluator = create_evaluator(game, evaluator_spec);
      let duration = convert_duration(time_per_move);
      Box::new(MinimaxAgent::new_boxed(evaluator, depth, duration))
    }
  }
}

fn create_regression<S: State, FE: FeatureExtractor<S>>(
  spec: &RegressionSpec,
  extractor: &FE,
) -> LinearRegressionTanh {
  if spec.params.len() == 0 {
    LinearRegressionTanh::zeros(extractor.nfeatures(), spec.regularization)
  } else {
    LinearRegressionTanh::new(spec.params.as_slice(), spec.regularization)
  }
}

pub fn create_evaluator<G: Game>(
  game: &'static G,
  spec: &EvaluatorSpec,
) -> Box<Evaluator<G::State>> {
  match spec {
    &EvaluatorSpec::Terminal => Box::new(TerminalEvaluator::new()),

    &EvaluatorSpec::Features {
      extractor: ref extractor_spec,
      regression: ref regression_spec,
    } => match extractor_spec {
      &FeatureExtractorSpec::Subtractor(nfeatures) => {
        let extractor = SubtractorFeatureExtractor::new(nfeatures);
        let regression = create_regression(regression_spec, &extractor);
        let subtractor: &Subtractor = (game as &Any).downcast_ref().unwrap();
        let evaluator =
          FeatureEvaluator::new(subtractor, extractor, regression);
        unsafe {
          transmute::<
            Box<Evaluator<<Subtractor as Game>::State>>,
            Box<Evaluator<G::State>>,
          >(Box::new(evaluator))
        }
      }
      &FeatureExtractorSpec::GomokuLines(min_len) => {
        let extractor = GomokuLineFeatureExtractor::new(min_len as usize);
        let regression = create_regression(regression_spec, &extractor);
        let gomoku: &Gomoku = (game as &Any).downcast_ref().unwrap();
        let evaluator = FeatureEvaluator::new(gomoku, extractor, regression);
        unsafe {
          transmute::<
            Box<Evaluator<<Gomoku as Game>::State>>,
            Box<Evaluator<G::State>>,
          >(Box::new(evaluator))
        }
      }
      &FeatureExtractorSpec::HexapawnNumberOfPawns => {
        let extractor = HexapawnNumberOfPawnsExtractor::new();
        let regression = create_regression(regression_spec, &extractor);
        let gomoku: &Hexapawn = (game as &Any).downcast_ref().unwrap();
        let evaluator = FeatureEvaluator::new(gomoku, extractor, regression);
        unsafe {
          transmute::<
            Box<Evaluator<<Hexapawn as Game>::State>>,
            Box<Evaluator<G::State>>,
          >(Box::new(evaluator))
        }
      }
    },

    &EvaluatorSpec::Sampler { samples, discount } => {
      Box::new(SamplerEvaluator::new(samples, discount))
    }
  }
}

pub fn create_training<G: Game>(
  game: &'static G,
  spec: &TrainingSpec,
) -> Box<Trainer<G>> {
  match &spec.extractor {
    &FeatureExtractorSpec::Subtractor(nfeatures) => {
      let extractor = SubtractorFeatureExtractor::new(nfeatures);
      let regression = create_regression(&spec.regression, &extractor);
      let subtractor: &Subtractor = (game as &Any).downcast_ref().unwrap();
      let trainer =
        create_trainer(subtractor, extractor, regression, &spec.trainer);
      unsafe { transmute::<Box<Trainer<Subtractor>>, Box<Trainer<G>>>(trainer) }
    }
    &FeatureExtractorSpec::GomokuLines(min_len) => {
      let extractor = GomokuLineFeatureExtractor::new(min_len as usize);
      let regression = create_regression(&spec.regression, &extractor);
      let gomoku: &Gomoku = (game as &Any).downcast_ref().unwrap();
      let trainer =
        create_trainer(gomoku, extractor, regression, &spec.trainer);
      unsafe { transmute::<Box<Trainer<Gomoku>>, Box<Trainer<G>>>(trainer) }
    }
    &FeatureExtractorSpec::HexapawnNumberOfPawns => {
      let extractor = HexapawnNumberOfPawnsExtractor::new();
      let regression = create_regression(&spec.regression, &extractor);
      let hexapawn: &Hexapawn = (game as &Any).downcast_ref().unwrap();
      let trainer =
        create_trainer(hexapawn, extractor, regression, &spec.trainer);
      unsafe { transmute::<Box<Trainer<Hexapawn>>, Box<Trainer<G>>>(trainer) }
    }
  }
}

fn create_trainer<
  G: Game,
  FE: FeatureExtractor<G::State> + Clone + 'static,
  R: Regression + 'static,
>(
  game: &'static G,
  extractor: FE,
  regression: R,
  spec: &TrainerSpec,
) -> Box<Trainer<G>> {
  match spec {
    &TrainerSpec::Reinforce {
      minimax_depth,
      random_prob,
      alpha,
    } => Box::new(ReinforceTrainer::new(
      game,
      extractor,
      regression,
      minimax_depth,
      random_prob,
      alpha,
    )),
    &TrainerSpec::Annealing {
      step_size,
      minimax_depth,
      temperature,
      ngames
    } => Box::new(AnnealingTrainer::new(
      game,
      extractor,
      regression,
      step_size,
      minimax_depth,
      temperature,
      ngames,
    )),
  }
}

fn convert_duration(seconds: f64) -> Option<Duration> {
  if seconds <= 0.0 {
    None
  } else {
    Some(Duration::new(
      seconds.trunc() as u64,
      (seconds.fract() * 1E9) as u32,
    ))
  }
}


#[cfg(test)]
mod test {

  use def::State;
  use games::{Gomoku, Hexapawn, Subtractor};
  use spec::*;
  use super::*;

  #[test]
  fn hexapawn_random() {
    let game = Hexapawn::default(3, 3);
    let agent_spec = AgentSpec::Random;
    let agent = create_agent(game, &agent_spec);
    let mut state = game.new_game();
    let report = agent.select_move(&state).unwrap();
    assert!(state.play(report.get_move()).is_ok())
  }

  #[test]
  fn hexapawn_terminal() {
    let game = Hexapawn::default(3, 3);
    let agent_spec = AgentSpec::Minimax {
      depth: 10,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::Terminal,
      name: String::new(),
    };
    let agent = create_agent(game, &agent_spec);
    let mut state = game.new_game();
    let report = agent.select_move(&state).unwrap();
    assert!(state.play(report.get_move()).is_ok());
    let report_str = format!("{}", report);
    assert!(report_str.contains("score"));
  }

  #[test]
  fn subtractor_features() {
    let agent_spec = AgentSpec::Minimax {
      depth: 3,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::Features {
        extractor: FeatureExtractorSpec::Subtractor(3),
        regression: RegressionSpec {
          regularization: 0.001,
          params: vec![0.1, 0.2, 0.3],
        },
      },
      name: String::new(),
    };

    let game = Subtractor::default(21, 4);
    let agent = create_agent(game, &agent_spec);
    let mut state = game.new_game();
    let report = agent.select_move(&state).unwrap();
    assert!(state.play(report.get_move()).is_ok())
  }

  #[test]
  fn gomoku_features() {
    let agent_spec = AgentSpec::Minimax {
      depth: 1,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::Features {
        extractor: FeatureExtractorSpec::GomokuLines(1),
        regression: RegressionSpec {
          params: vec![],
          regularization: 0.001,
        },
      },
      name: String::new(),
    };

    let game = Gomoku::default();
    let agent = create_agent(game, &agent_spec);
    let mut state = game.new_game();
    let report = agent.select_move(&state).unwrap();
    assert!(state.play(report.get_move()).is_ok())
  }

  #[test]
  fn hexapawn_features() {
    let agent_spec = AgentSpec::Minimax {
      depth: 2,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::Features {
        extractor: FeatureExtractorSpec::HexapawnNumberOfPawns,
        regression: RegressionSpec {
          params: vec![0.0, 1.0, -1.0],
          regularization: 0.001,
        },
      },
      name: String::new(),
    };

    let game = Hexapawn::default(3, 3);
    let agent = create_agent(game, &agent_spec);
    let mut state = game.new_game();
    let report = agent.select_move(&state).unwrap();
    assert!(state.play(report.get_move()).is_ok())
  }

  #[test]
  fn subtractor_training() {
    let game = Subtractor::default(21, 4);

    let training_spec = TrainingSpec {
      extractor: FeatureExtractorSpec::Subtractor(5),
      regression: RegressionSpec {
        params: vec![],
        regularization: 0.001,
      },
      trainer: TrainerSpec::Reinforce {
        minimax_depth: 5,
        random_prob: 0.1,
        alpha: 0.001,
      },
    };

    let mut trainer = create_training(game, &training_spec);

    trainer.train(100, Duration::new(0, 0));
    trainer.build_evaluator();
  }

} // mod test
