use std::any::Any;
use std::mem::transmute;
use std::time::Duration;

use def::{Agent, Evaluator, Game};
use feature_evaluator::{FeatureEvaluator, LinearRegression, Regression};
use gomoku::{Gomoku, GomokuLineFeatureExtractor, GomokuState};
use hexapawn::{Hexapawn, HexapawnNumberOfPawnsExtractor, HexapawnState};
use human_agent::HumanAgent;
use minimax::MinimaxAgent;
use random_agent::RandomAgent;
use spec::{AgentSpec, EvaluatorSpec, FeatureExtractorSpec};
use subtractor::{Subtractor, SubtractorFeatureExtractor, SubtractorState};
use terminal_evaluator::TerminalEvaluator;


pub fn create_agent<G: Game>(game: &'static G, spec: &AgentSpec)
    -> Box<Agent<G::State>> {
  match spec {
    &AgentSpec::Random => Box::new(RandomAgent::new()),

    &AgentSpec::Human => Box::new(HumanAgent{}),

    &AgentSpec::Minimax{
      depth,
      time_per_move,
      evaluator: ref evaluator_spec
    } => {
      let evaluator = create_evaluator(game, evaluator_spec);
      let duration = convert_duration(time_per_move);
      Box::new(MinimaxAgent::new_boxed(evaluator, depth, duration))
    }
  }
}

pub fn create_evaluator<G: Game>(game: &'static G, spec: &EvaluatorSpec)
    -> Box<Evaluator<G::State>> {
  match spec {
    &EvaluatorSpec::Terminal => Box::new(TerminalEvaluator::new()),

    &EvaluatorSpec::Features{
      extractor: ref extractor_spec,
      regression: ref regression_spec,
      training_minimax_depth,
      steps
    } => {
      let mut regression = LinearRegression::new(
          regression_spec.b.clone(),
          (regression_spec.speed, regression_spec.regularization));
      match extractor_spec {
        &FeatureExtractorSpec::Subtractor(nfeatures) => {
          let extractor = SubtractorFeatureExtractor::new(nfeatures);
          regression.init(&extractor);
          let subtractor: &Subtractor =
              (game as &Any).downcast_ref().unwrap();
          let evaluator = FeatureEvaluator::new(
              subtractor, extractor, regression, training_minimax_depth, steps);
          unsafe{ transmute::<Box<Evaluator<SubtractorState>>,
                              Box<Evaluator<G::State>>>(Box::new(evaluator)) }
        },
        &FeatureExtractorSpec::GomokuLines => {
          let extractor = GomokuLineFeatureExtractor::new();
          regression.init(&extractor);
          let gomoku: &Gomoku = (game as &Any).downcast_ref().unwrap();
          let evaluator = FeatureEvaluator::new(gomoku, extractor, regression, training_minimax_depth, steps);
          unsafe{ transmute::<Box<Evaluator<GomokuState>>,
                              Box<Evaluator<G::State>>>(Box::new(evaluator)) }
        },
        &FeatureExtractorSpec::HexapawnNumberOfPawns => {
          let extractor = HexapawnNumberOfPawnsExtractor::new();
          regression.init(&extractor);
          let gomoku: &Hexapawn = (game as &Any).downcast_ref().unwrap();
          let evaluator = FeatureEvaluator::new(gomoku, extractor, regression, training_minimax_depth, steps);
          unsafe{ transmute::<Box<Evaluator<HexapawnState>>,
                              Box<Evaluator<G::State>>>(Box::new(evaluator)) }
        },
      }
    }
  }
}

fn convert_duration(seconds: f64) -> Option<Duration> {
  if seconds <= 0.0 {
    None
  } else {
    Some(Duration::new(seconds.trunc() as u64,
                       (seconds.fract() * 1E9) as u32))
  }
}


#[cfg(test)]
mod test {

use def::State;
use gomoku::Gomoku;
use hexapawn::Hexapawn;
use spec::*;
use subtractor::Subtractor;
use super::*;

#[test]
fn hexapawn_random() {
  let game = Hexapawn::default(3, 3);
  let agent_spec = AgentSpec::Random;
  let mut agent = create_agent(game, &agent_spec);
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
    evaluator: EvaluatorSpec::Terminal
  };
  let mut agent = create_agent(game, &agent_spec);
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
        speed: 0.001,
        regularization: 0.001,
        b: vec![0.1, 0.2, 0.3]
      },
      training_minimax_depth: 1,
      steps: 0
    }
  };

  let game = Subtractor::default(21, 4);
  let mut agent = create_agent(game, &agent_spec);
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
      extractor: FeatureExtractorSpec::GomokuLines,
      regression: RegressionSpec {
        speed: 0.001,
        regularization: 0.001,
        b: vec![]
      },
      training_minimax_depth: 1,
      steps: 0
    }
  };

  let game = Gomoku::default();
  let mut agent = create_agent(game, &agent_spec);
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
        speed: 0.001,
        regularization: 0.001,
        b: vec![0.0, 1.0, -1.0]
      },
      training_minimax_depth: 1,
      steps: 0
    }
  };

  let game = Hexapawn::default(3, 3);
  let mut agent = create_agent(game, &agent_spec);
  let mut state = game.new_game();
  let report = agent.select_move(&state).unwrap();
  assert!(state.play(report.get_move()).is_ok())
}

}  // mod test