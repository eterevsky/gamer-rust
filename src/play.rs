use std::borrow::BorrowMut;
use std::time::Duration;

use def::{Agent, Evaluator, Game, State};
use feature_evaluator::{FeatureExtractor, FeatureEvaluator, LinearRegression, Regression};
use gomoku::{Gomoku, GomokuLineFeatureExtractor, GomokuState};
use hexapawn::Hexapawn;
use minimax::MinimaxAgent;
use random_agent::RandomAgent;
use spec::{GameSpec, AgentSpec, EvaluatorSpec, MinimaxSpec, FeatureExtractorSpec};
use subtractor::{Subtractor, SubtractorFeatureExtractor, SubtractorState};
use terminal_evaluator::TerminalEvaluator;

pub fn play<'g, G: Game<'g>>(
    game: &'g G,
    mut player1: Box<Agent<'g, G::State> + 'g>,
    mut player2: Box<Agent<'g, G::State> + 'g>) -> f32 {
  let mut state = game.new_game();

  while !state.is_terminal() {
    println!("{}", state);
    let player: &mut Agent<'g, G::State> =
        if state.get_player() { player1.borrow_mut() }
        else { player2.borrow_mut() };
    let report = player.select_move(&state).unwrap();
    println!("Player {}: {}\n", if state.get_player() { 1 } else { 2 }, report);
    state.play(report.get_move()).unwrap();
  }

  println!("{}", state);
  let payoff = state.get_payoff().unwrap();
  println!("Payoff: {}", payoff);
  payoff
}

trait Creating<'g, G: Game<'g>> {
  fn agent(game: &'g G, spec: &AgentSpec)
      -> Box<Agent<'g, G::State> + 'g> {
    match spec {
      &AgentSpec::Random => Box::new(RandomAgent::new()),
      &AgentSpec::Human => panic!("Human player not yet implemented."),
      &AgentSpec::Minimax(ref minimax_spec) => {
        match minimax_spec.evaluator {
          EvaluatorSpec::TerminalEvaluator =>
              Self::minimax_agent(game, minimax_spec, TerminalEvaluator::new()),
          EvaluatorSpec::FeatureEvaluator(ref feature_evaluator_spec) => {
              Self::minimax_feature_extractor(
                  game, minimax_spec, &feature_evaluator_spec.extractor)
          }
        }
      }
    }
  }

  fn minimax_agent<E: Evaluator<'g, G::State> + 'g>(
      _game: &'g G, spec: &MinimaxSpec, evaluator: E)
      -> Box<Agent<'g, G::State> + 'g> {
    Box::new(MinimaxAgent::new(evaluator, spec.depth, Duration::from_secs(1)))
  }

  fn minimax_feature_extractor(
      _game: &'g G, _minimax_spec: &MinimaxSpec, _spec: &FeatureExtractorSpec)
      -> Box<Agent<'g, G::State> + 'g> {
    panic!("Feature extractors are not defined for this game.")
  }

  fn minimax_agent_with_extractor<FE>(
      game: &'g G, spec: &MinimaxSpec, extractor: FE)
      -> Box<Agent<'g, G::State> + 'g>
      where FE: FeatureExtractor<'g, G::State, FeatureVector=Vec<f32>> + 'g {

    if let EvaluatorSpec::FeatureEvaluator(ref fe_spec) = spec.evaluator {
      let regression = LinearRegression::new(
          fe_spec.regression.b.clone(),
          (fe_spec.regression.speed, fe_spec.regression.regularization));
      Self::minimax_agent(
          game, spec, FeatureEvaluator::new(game, extractor, regression))
    } else {
      panic!("Internal error")
    }
  }
}

struct Creator {}

impl<'g> Creating<'g, Subtractor> for Creator {
  fn minimax_feature_extractor(
      game: &'g Subtractor, minimax_spec: &MinimaxSpec,
      spec: &FeatureExtractorSpec)
      -> Box<Agent<'g, SubtractorState> + 'g> {
    let extractor = match spec {
      &FeatureExtractorSpec::SubtractorFeatureExtractor(nfeatures) =>
          SubtractorFeatureExtractor::new(nfeatures),
      _ => panic!("Invalid feature extractor for Subtractor game: {:?}", spec)
    };
    Self::minimax_agent_with_extractor(game, minimax_spec, extractor)
  }
}

impl<'g> Creating<'g, Gomoku<'g>> for Creator {
  fn minimax_feature_extractor(
      game: &'g Gomoku<'g>, minimax_spec: &MinimaxSpec,
      spec: &FeatureExtractorSpec)
      -> Box<Agent<'g, GomokuState<'g>> + 'g> {
    let extractor = match spec {
      &FeatureExtractorSpec::GomokuLineFeatureExtractor =>
          GomokuLineFeatureExtractor::new(),
      _ => panic!("Invalid feature extractor for gomoku: {:?}", spec)
    };
    Self::minimax_agent_with_extractor(game, minimax_spec, extractor)
  }
}

impl<'g> Creating<'g, Hexapawn> for Creator {
}

pub fn play_spec(
    game_spec: &GameSpec, player1_spec: &AgentSpec, player2_spec: &AgentSpec)
    -> f32 {
  match game_spec {
    &GameSpec::Gomoku => {
      let game = Gomoku::default();
      let player1 = Creator::agent(game, player1_spec);
      let player2 = Creator::agent(game, player2_spec);
      let payoff = play(game, player1, player2);
      payoff
    },
    &GameSpec::Hexapawn(width, height) => {
      let game = Hexapawn::new(width, height);
      let player1 = Creator::agent(&game, player1_spec);
      let player2 = Creator::agent(&game, player2_spec);
      let payoff = play(&game, player1, player2);
      payoff
    },
    &GameSpec::Subtractor(start, max_sub) => {
      let game = Subtractor::new(start, max_sub);
      let player1 = Creator::agent(&game, player1_spec);
      let player2 = Creator::agent(&game, player2_spec);
      let payoff = play(&game, player1, player2);
      payoff
    }
  }
}

#[cfg(test)]
mod test {

use spec::*;
use super::*;

#[test]
fn agent_from_spec() {
  let game_spec = GameSpec::Subtractor(21, 4);

  let agent1_spec = AgentSpec::Minimax(MinimaxSpec {
      depth: 1,
      evaluator: EvaluatorSpec::FeatureEvaluator(FeatureEvaluatorSpec {
        extractor: FeatureExtractorSpec::SubtractorFeatureExtractor(3),
        regression: RegressionSpec {
          speed: 0.001,
          regularization: 0.001,
          b: vec![0.1, 0.2, 0.3]
        }
      })
  });

  let agent2_spec = AgentSpec::Minimax(MinimaxSpec {
      depth: 3,
      evaluator: EvaluatorSpec::TerminalEvaluator
  });

  assert_eq!(-1.0, play_spec(&game_spec, &agent1_spec, &agent2_spec));
}

}  // mod test