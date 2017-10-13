use std::borrow::BorrowMut;
use std::time::Duration;

use def::{Agent, Evaluator, Game, State};
use feature_evaluator::{FeatureExtractor, FeatureEvaluator, LinearRegression, Regression};
use gomoku::{Gomoku, GomokuLineFeatureExtractor, GomokuState};
use hexapawn::{Hexapawn, HexapawnState};
use minimax::MinimaxAgent;
use random_agent::RandomAgent;
use spec::{GameSpec, AgentSpec, EvaluatorSpec, MinimaxSpec, FeatureExtractorSpec};
use subtractor::{Subtractor, SubtractorFeatureExtractor, SubtractorState};
use terminal_evaluator::TerminalEvaluator;

pub fn play<'g, G: Game<'g>>(
    game: &'g G,
    mut player1: Box<Agent<'g, G::State> + 'g>,
    mut player2: Box<Agent<'g, G::State> + 'g>) {
  let mut state = game.new_game();

  while !state.is_terminal() {
    println!("{}", state);
    let player: &mut Agent<'g, G::State> =
        if state.get_player() { player1.borrow_mut() }
        else { player2.borrow_mut() };
    let report = player.select_move(&state).unwrap();
    println!("Player {}:", if state.get_player() { 1 } else { 2 });
    println!("{}\n", report);
    state.play(report.get_move()).unwrap();
  }

  println!("{}", state);
  println!("Payoff: {}", state.get_payoff().unwrap());
}

trait MinimaxFeatureEvaluatorCreator<'g, G: Game<'g>> {
  fn create_agent(game: &'g G, spec: &AgentSpec)
    -> Box<Agent<'g, G::State> + 'g> {
    match spec {
      &AgentSpec::Random => Box::new(RandomAgent::new()),
      &AgentSpec::Human => panic!("Human player not yet implemented."),
      &AgentSpec::Minimax(ref mspec) => {
        match mspec.evaluator {
          EvaluatorSpec::TerminalEvaluator => {
            Box::new(MinimaxAgent::new(
                TerminalEvaluator::new(), mspec.depth, Duration::from_secs(1)))
          },
          EvaluatorSpec::FeatureEvaluator(ref fe_spec) => {
            let regression = LinearRegression::new(
                fe_spec.regression.b.clone(),
                (fe_spec.regression.speed, fe_spec.regression.regularization));
            Self::create(
                game, regression, mspec, &fe_spec.extractor)
          }
        }
      }
    }
  }

  fn create<R: Regression<Vec<f32>> + 'g>(
      _game: &'g G, regression: R, mspec: &MinimaxSpec, fspec: &FeatureExtractorSpec)
      -> Box<Agent<'g, G::State> + 'g> {
    panic!()
  }

  fn create_minimax_agent<R, FE>(game: &'g G, mspec: &MinimaxSpec, regression: R, extractor: FE)
      -> Box<Agent<'g, G::State> + 'g>
      where R: Regression<Vec<f32>> + 'g,
            FE: FeatureExtractor<'g, G::State, FeatureVector=Vec<f32>> + 'g {
    Box::new(MinimaxAgent::new(
        FeatureEvaluator::new(game, extractor, regression),
        mspec.depth,
        Duration::from_secs(1)
    ))
  }
}

struct FeatureExtractorCreator {}

impl<'g> MinimaxFeatureEvaluatorCreator<'g, Subtractor> for FeatureExtractorCreator {
  fn create<R: Regression<Vec<f32>> + 'g>(game: &'g Subtractor, regression: R,
                                   mspec: &MinimaxSpec, fspec: &FeatureExtractorSpec)
      -> Box<Agent<'g, SubtractorState> + 'g> {
    let extractor = match fspec {
      &FeatureExtractorSpec::SubtractorFeatureExtractor(nfeatures) =>
          SubtractorFeatureExtractor::new(nfeatures),
      _ => panic!()
    };
    Self::create_minimax_agent(game, mspec, regression, extractor)
  }
}

impl<'g> MinimaxFeatureEvaluatorCreator<'g, Gomoku<'g>> for FeatureExtractorCreator {
  fn create<R: Regression<Vec<f32>> + 'g>(game: &'g Gomoku<'g>, regression: R,
                                   mspec: &MinimaxSpec, fspec: &FeatureExtractorSpec)
      -> Box<Agent<'g, GomokuState<'g>> + 'g> {
    let extractor = match fspec {
      &FeatureExtractorSpec::GomokuLineFeatureExtractor =>
          GomokuLineFeatureExtractor::new(),
      _ => panic!()
    };
    Self::create_minimax_agent(game, mspec, regression, extractor)
  }
}

impl<'g> MinimaxFeatureEvaluatorCreator<'g, Hexapawn> for FeatureExtractorCreator {
}

pub fn create_agent_gomoku<'g>(game: &'g Gomoku<'g>, spec: &AgentSpec)
    -> Box<Agent<'g, GomokuState<'g>> + 'g>  {
  FeatureExtractorCreator::create_agent(game, spec)
}

pub fn create_agent_hexapawn<'g>(game: &'g Hexapawn, spec: &AgentSpec)
    -> Box<Agent<'g, HexapawnState> + 'g>  {
  FeatureExtractorCreator::create_agent(game, spec)
}

pub fn create_agent_subtractor<'g>(game: &'g Subtractor, spec: &AgentSpec)
    -> Box<Agent<'g, SubtractorState> + 'g>  {
  FeatureExtractorCreator::create_agent(game, spec)
}

pub fn play_spec(game_spec: &GameSpec, player1_spec: &AgentSpec, player2_spec: &AgentSpec) {
  match game_spec {
    &GameSpec::Gomoku => {
      let game = Gomoku::default();
      let player1 = create_agent_gomoku(game, player1_spec);
      let player2 = create_agent_gomoku(game, player2_spec);
      play(game, player1, player2);
    },
    &GameSpec::Hexapawn(width, height) => {
      let game = Hexapawn::new(width, height);
      let player1 = create_agent_hexapawn(&game, player1_spec);
      let player2 = create_agent_hexapawn(&game, player2_spec);
      play(&game, player1, player2);
    },
    &GameSpec::Subtractor(start, max_sub) => {
      let game = Subtractor::new(start, max_sub);
      let player1 = create_agent_subtractor(&game, player1_spec);
      let player2 = create_agent_subtractor(&game, player2_spec);
      play(&game, player1, player2);
    }
  }
}

#[cfg(test)]
mod test {

use spec::*;
use super::*;

#[test]
fn create_agent_from_spec() {
  let agent_spec = AgentSpec::Minimax(MinimaxSpec {
      depth: 3,
      evaluator: EvaluatorSpec::FeatureEvaluator(FeatureEvaluatorSpec {
        extractor: FeatureExtractorSpec::SubtractorFeatureExtractor(10),
        regression: RegressionSpec {
          speed: 0.001,
          regularization: 0.001,
          b: vec![0.1, 0.2, 0.3]
        }
      })
  });

  let game = Subtractor::new(21, 4);
  let agent = create_agent_subtractor(&game, &agent_spec);
}

}  // mod test