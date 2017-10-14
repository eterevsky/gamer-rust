use std::time::Duration;

use def::{Agent, Game};
// use feature_evaluator::{FeatureEvaluator, LinearRegression, Regression};
// use gomoku::{Gomoku, GomokuLineFeatureExtractor, GomokuState};
// use hexapawn::{Hexapawn, HexapawnState};
// use human_agent::HumanAgent;
// use minimax::MinimaxAgent;
use random_agent::RandomAgent;
use spec::AgentSpec;
// use subtractor::{Subtractor, SubtractorFeatureExtractor, SubtractorState};
// use terminal_evaluator::TerminalEvaluator;


pub fn create_agent<G: Game>(_game: &G, player_spec: &AgentSpec) -> Box<Agent<G::State>> {
  Box::new(RandomAgent::new())
}

fn convert_duration(seconds: f64) -> Option<Duration> {
  if seconds <= 0.0 {
    None
  } else {
    Some(Duration::new(seconds.trunc() as u64, (seconds.fract() * 1E9) as u32))
  }
}

// trait Creating<'g, G: Game<'g>> {
//   fn agent(game: &'g G, spec: &AgentSpec)
//       -> Box<Agent<'g, G::State> + 'g> {
//     match spec {
//       &AgentSpec::Random => Box::new(RandomAgent::new()),

//       &AgentSpec::Human => Box::new(HumanAgent{}),

//       &AgentSpec::Minimax(ref minimax_spec) => {
//         let evaluator: Box<Evaluator<'g, G::State> + 'g> =
//             match minimax_spec.evaluator {
//               EvaluatorSpec::TerminalEvaluator =>
//                   Box::new(TerminalEvaluator::new()),
//               _ => Self::training_evaluator(game, &minimax_spec.evaluator)
//             };
//         let duration = convert_duration(minimax_spec.time_per_move);
//         Box::new(MinimaxAgent::new(evaluator, minimax_spec.depth, duration))
//       }
//     }
//   }

//   fn training_evaluator(game: &'g G, spec: &EvaluatorSpec)
//       -> Box<TrainingEvaluator<'g, G::State> + 'g> {
//     match spec {
//       &EvaluatorSpec::FeatureEvaluator(ref fespec) => {
//         Self::evaluator_feature_extractor(game, spec, fespec)
//       },
//       _ => unreachable!()
//     }
//   }

//   fn evaluator_feature_extractor(
//       game: &'g G, evaluator_spec: &EvaluatorSpec, spec: &FeatureExtractorSpec)
//       -> Box<TrainingEvaluator<'g, G::State> + 'g>;

//   fn evaluator_with_extractor<FE>(
//       game: &'g G, spec: &EvaluatorSpec, extractor: FE)
//       -> Box<TrainingEvaluator<'g, G::State> + 'g>
//       where FE: FeatureExtractor<'g, G::State, FeatureVector=Vec<f32>> + 'g {
//     if let EvaluatorSpec::FeatureEvaluator(ref fe_spec) = spec.evaluator {
//       let regression = LinearRegression::new(
//           fe_spec.regression.b.clone(),
//           (fe_spec.regression.speed, fe_spec.regression.regularization));
//       FeatureEvaluator::new(game, extractor, regression)
//     } else {
//       unreachable!()
//     }
//   }
// }

// struct Creator {}

// impl<'g> Creating<'g, Gomoku<'g>> for Creator {
//   fn evaluator_feature_extractor(
//       game: &'g Gomoku<'g>, evaluator_spec: &EvaluatorSpec,
//       spec: &FeatureExtractorSpec)
//       -> Box<TrainingEvaluator<'g, GomokuState<'g>> + 'g> {
//     match spec {
//       &FeatureExtractorSpec::GomokuLineFeatureExtractor => {
//           let extractor = GomokuLineFeatureExtractor::new();
//           Self::evaluator_with_extractor(game, evaluator_spec, extractor)
//       },
//       _ => panic!("Invalid feature extractor for Gomoku game: {:?}", spec)
//     };
//   }
// }

// impl<'g> Creating<'g, Hexapawn> for Creator {
//   fn evaluator_feature_extractor(
//       game: &'g Hexapawn, evaluator_spec: &EvaluatorSpec,
//       spec: &FeatureExtractorSpec)
//       -> Box<TrainingEvaluator<'g, HexapawnState> + 'g> {
//     panic!("Invalid feature extractor for Hexapawn game: {:?}", spec)
//   }
// }

// impl<'g> Creating<'g, Subtractor> for Creator {
//   fn evaluator_feature_extractor(
//       game: &'g Subtractor, evaluator_spec: &EvaluatorSpec,
//       spec: &FeatureExtractorSpec)
//       -> Box<TrainingEvaluator<'g, SubtractorState> + 'g> {
//     match spec {
//       &FeatureExtractorSpec::SubtractorFeatureExtractor(nfeatures) => {
//           let extractor = SubtractorFeatureExtractor::new(nfeatures);
//           Self::evaluator_with_extractor(game, evaluator_spec, extractor)
//       },
//       _ => panic!("Invalid feature extractor for Subtractor game: {:?}", spec)
//     };
//   }
// }


// #[cfg(test)]
// mod test {

// use spec::*;
// use super::*;

// #[test]
// fn agent_from_spec() {
//   let game_spec = GameSpec::Subtractor(21, 4);

//   let agent1_spec = AgentSpec::Minimax(MinimaxSpec {
//       depth: 1,
//       time_per_move: 0.0,
//       evaluator: EvaluatorSpec::FeatureEvaluator(FeatureEvaluatorSpec {
//         extractor: FeatureExtractorSpec::SubtractorFeatureExtractor(3),
//         regression: RegressionSpec {
//           speed: 0.001,
//           regularization: 0.001,
//           b: vec![0.1, 0.2, 0.3]
//         }
//       })
//   });

//   let agent2_spec = AgentSpec::Minimax(MinimaxSpec {
//       depth: 3,
//       time_per_move: 0.0,
//       evaluator: EvaluatorSpec::TerminalEvaluator
//   });

//   assert_eq!(-1.0, play_spec(&game_spec, &agent1_spec, &agent2_spec, 0.0));
// }

// }  // mod test