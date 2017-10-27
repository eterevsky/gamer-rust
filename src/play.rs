use std::time::Duration;

use def::{Agent, Game, State};
use gomoku::Gomoku;
use hexapawn::Hexapawn;
use minimax::MinimaxAgent;
use registry::{create_agent, create_evaluator};
use spec::{AgentSpec, EvaluatorSpec, GameSpec};
use subtractor::Subtractor;

pub fn play<G: Game>(
    game: &G, player1: &mut Agent<G::State>, player2: &mut Agent<G::State>
) -> f32 {
  let mut state = game.new_game();

  while !state.is_terminal() {
    println!("{}", state);
    let report = if state.get_player() {
      player1.select_move(&state).unwrap()
    } else {
      player2.select_move(&state).unwrap()
    };
    println!("{}", report);
    state.play(report.get_move()).unwrap();
  }

  println!("{}", state);
  let payoff = state.get_payoff().unwrap();
  println!("Payoff: {}", payoff);
  payoff
}

pub fn play_game<G: Game>(
    game: &'static G, player1_spec: &AgentSpec, player2_spec: &AgentSpec
) -> f32 {
  let mut player1 = create_agent(game, player1_spec);
  let mut player2 = create_agent(game, player2_spec);
  let payoff = play(game, &mut *player1, &mut *player2);
  payoff
}

pub fn play_spec(
    game_spec: &GameSpec, player1_spec: &AgentSpec, player2_spec: &AgentSpec
) -> f32 {
  match game_spec {
    &GameSpec::Gomoku => {
      let game = Gomoku::default();
      play_game(game, player1_spec, player2_spec)
    },
    &GameSpec::Hexapawn(width, height) => {
      let game = Hexapawn::default(width, height);
      play_game(game, player1_spec, player2_spec)
    },
    &GameSpec::Subtractor(start, max_sub) => {
      let game = Subtractor::default(start, max_sub);
      play_game(game, player1_spec, player2_spec)
    }
  }
}

pub fn train_game<G: Game>(
    game: &'static G, evaluator_spec: &EvaluatorSpec, steps: u64,
    time_limit: Duration
) -> AgentSpec {
  let mut evaluator = create_evaluator(game, evaluator_spec);
  evaluator.train(steps, time_limit);
  let minimax = MinimaxAgent::new_boxed(evaluator, 1000, None);
  minimax.spec()
}

pub fn train_spec(
    game_spec: &GameSpec, evaluator_spec: &EvaluatorSpec, steps: u64,
    time_limit: Duration
) -> AgentSpec {
  match game_spec {
    &GameSpec::Gomoku => {
      let game = Gomoku::default();
      train_game(game, evaluator_spec, steps, time_limit)
    },
    &GameSpec::Hexapawn(width, height) => {
      let game = Hexapawn::default(width, height);
      train_game(game, evaluator_spec, steps, time_limit)
    },
    &GameSpec::Subtractor(start, max_sub) => {
      let game = Subtractor::default(start, max_sub);
      train_game(game, evaluator_spec, steps, time_limit)
    }
  }
}

#[cfg(test)]
mod test {

use spec::*;
use super::*;

#[test]
fn play_hexapawn() {
  let game_spec = GameSpec::Hexapawn(3, 3);
  let agent1_spec = AgentSpec::Minimax {
    depth: 3,
    time_per_move: 0.0,
    evaluator: EvaluatorSpec::Terminal
  };
  let agent2_spec = AgentSpec::Minimax {
    depth: 10,
    time_per_move: 0.0,
    evaluator: EvaluatorSpec::Terminal
  };
  assert_eq!(-1.0, play_spec(&game_spec, &agent1_spec, &agent2_spec));
}

}
