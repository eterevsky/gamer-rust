use std::borrow::BorrowMut;
use std::time::Duration;

use def::{Agent, Evaluator, Game, State};
use feature_evaluator::{FeatureExtractor, FeatureEvaluator, LinearRegression, Regression};
use gomoku::{Gomoku, GomokuLineFeatureExtractor, GomokuState};
use hexapawn::Hexapawn;
use human_agent::HumanAgent;
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
    println!("{}", report);
    state.play(report.get_move()).unwrap();
  }

  println!("{}", state);
  let payoff = state.get_payoff().unwrap();
  println!("Payoff: {}", payoff);
  payoff
}

trait Creating<'g, G: Game<'g>> {
  fn agent(&self, game: &'g G, spec: &AgentSpec)
      -> Box<Agent<'g, G::State> + 'g> {
    match spec {
      &AgentSpec::Random => Box::new(RandomAgent::new()),
      &AgentSpec::Human => Box::new(HumanAgent{}),
      &AgentSpec::Minimax(ref minimax_spec) => {
        match minimax_spec.evaluator {
          EvaluatorSpec::TerminalEvaluator =>
              self.minimax_agent(game, minimax_spec, TerminalEvaluator::new()),
          EvaluatorSpec::FeatureEvaluator(ref feature_evaluator_spec) => {
              self.minimax_feature_extractor(
                  game, minimax_spec, &feature_evaluator_spec.extractor)
          }
        }
      }
    }
  }

  fn get_time_per_move(&self) -> f64 {
    0.0
  }

  fn minimax_agent<E: Evaluator<'g, G::State> + 'g>(
      &self, _game: &'g G, spec: &MinimaxSpec, evaluator: E)
      -> Box<Agent<'g, G::State> + 'g> {
    let t = if spec.time_per_move == 0.0 {
      self.get_time_per_move()
    } else {
      spec.time_per_move
    };
    let duration = if t == 0.0 {
      None
    } else {
      Some(Duration::new(t.trunc() as u64, (t.fract() * 1E9) as u32))
    };
    Box::new(MinimaxAgent::new(evaluator, spec.depth, duration))
  }

  fn minimax_feature_extractor(
      &self, _game: &'g G, _minimax_spec: &MinimaxSpec, _spec: &FeatureExtractorSpec)
      -> Box<Agent<'g, G::State> + 'g> {
    panic!("Feature extractors are not defined for this game.")
  }

  fn minimax_agent_with_extractor<FE>(
      &self, game: &'g G, spec: &MinimaxSpec, extractor: FE)
      -> Box<Agent<'g, G::State> + 'g>
      where FE: FeatureExtractor<'g, G::State, FeatureVector=Vec<f32>> + 'g {

    if let EvaluatorSpec::FeatureEvaluator(ref fe_spec) = spec.evaluator {
      let regression = LinearRegression::new(
          fe_spec.regression.b.clone(),
          (fe_spec.regression.speed, fe_spec.regression.regularization));
      self.minimax_agent(
          game, spec, FeatureEvaluator::new(game, extractor, regression))
    } else {
      panic!("Internal error")
    }
  }
}

struct Creator {
  time_per_move: f64
}

impl<'g> Creating<'g, Subtractor> for Creator {
  fn get_time_per_move(&self) -> f64 {
    self.time_per_move
  }

  fn minimax_feature_extractor(
      &self, game: &'g Subtractor, minimax_spec: &MinimaxSpec,
      spec: &FeatureExtractorSpec)
      -> Box<Agent<'g, SubtractorState> + 'g> {
    let extractor = match spec {
      &FeatureExtractorSpec::SubtractorFeatureExtractor(nfeatures) =>
          SubtractorFeatureExtractor::new(nfeatures),
      _ => panic!("Invalid feature extractor for Subtractor game: {:?}", spec)
    };
    self.minimax_agent_with_extractor(game, minimax_spec, extractor)
  }
}

impl<'g> Creating<'g, Gomoku<'g>> for Creator {
  fn get_time_per_move(&self) -> f64 {
    self.time_per_move
  }

  fn minimax_feature_extractor(
      &self,
      game: &'g Gomoku<'g>, minimax_spec: &MinimaxSpec,
      spec: &FeatureExtractorSpec)
      -> Box<Agent<'g, GomokuState<'g>> + 'g> {
    let extractor = match spec {
      &FeatureExtractorSpec::GomokuLineFeatureExtractor =>
          GomokuLineFeatureExtractor::new(),
      _ => panic!("Invalid feature extractor for gomoku: {:?}", spec)
    };
    self.minimax_agent_with_extractor(game, minimax_spec, extractor)
  }
}

impl<'g> Creating<'g, Hexapawn> for Creator {
  fn get_time_per_move(&self) -> f64 {
    self.time_per_move
  }
}

// time_per_move will be applied to all agents that a) support it, b) do not
// have time limit in the spec.
pub fn play_spec(
    game_spec: &GameSpec, player1_spec: &AgentSpec, player2_spec: &AgentSpec,
    time_per_move: f64)
    -> f32 {
  let creator = Creator{ time_per_move };
  match game_spec {
    &GameSpec::Gomoku => {
      let game = Gomoku::default();
      let player1 = creator.agent(game, player1_spec);
      let player2 = creator.agent(game, player2_spec);
      let payoff = play(game, player1, player2);
      payoff
    },
    &GameSpec::Hexapawn(width, height) => {
      let game = Hexapawn::new(width, height);
      let player1 = creator.agent(&game, player1_spec);
      let player2 = creator.agent(&game, player2_spec);
      let payoff = play(&game, player1, player2);
      payoff
    },
    &GameSpec::Subtractor(start, max_sub) => {
      let game = Subtractor::new(start, max_sub);
      let player1 = creator.agent(&game, player1_spec);
      let player2 = creator.agent(&game, player2_spec);
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
      time_per_move: 0.0,
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
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::TerminalEvaluator
  });

  assert_eq!(-1.0, play_spec(&game_spec, &agent1_spec, &agent2_spec, 0.0));
}

}  // mod test