use std::borrow::BorrowMut;

use def::{Agent, Game, State};
use gomoku::Gomoku;
use random_agent::RandomAgent;
use subtractor::Subtractor;
use hexapawn::Hexapawn;

pub enum GameSpec {
  Gomoku,
  Hexapawn(u32, u32),
  Subtractor(u32, u32)
}

impl GameSpec {
  pub fn parse(s: &str) -> Option<GameSpec> {
    match s {
      "gomoku" => Some(GameSpec::Gomoku),
      "hexapawn" => Some(GameSpec::Hexapawn(8, 8)),
      "subtractor" => Some(GameSpec::Subtractor(21, 4)),
      _ => None
    }
  }
}

pub enum AgentSpec {
  Random,
  Human,
  Minimax
}

impl AgentSpec {
  pub fn parse(s: &str) -> Option<AgentSpec> {
    if s == "random" {
      Some(AgentSpec::Random)
    } else {
      None
    }
  }
}

pub fn play<'g, G: Game<'g>>(
    game: &'g G,
    mut player1: Box<Agent<'g, G::State>>,
    mut player2: Box<Agent<'g, G::State>>) {
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

fn create_agent<'g, G: Game<'g>>(_game: &'g G, spec: &AgentSpec)
    -> Box<Agent<'g, G::State>> {
  match spec {
    &AgentSpec::Random => Box::new(RandomAgent::new()),
    _ => panic!()
  }
}

fn play_agent_spec<'g, G: Game<'g>>(
    game: &'g G,
    player1_spec: &AgentSpec,
    player2_spec: &AgentSpec) {
  let player1 = create_agent(game, player1_spec);
  let player2 = create_agent(game, player2_spec);
  play(game, player1, player2);
}

pub fn play_spec(game_spec: &GameSpec, player1_spec: &AgentSpec, player2_spec: &AgentSpec) {
  match game_spec {
    &GameSpec::Gomoku =>
        play_agent_spec(Gomoku::default(), player1_spec, player2_spec),
    &GameSpec::Hexapawn(width, height) =>
        play_agent_spec(&Hexapawn::new(width, height), player1_spec, player2_spec),
    &GameSpec::Subtractor(start, max_sub) =>
        play_agent_spec(&Subtractor::new(start, max_sub), player1_spec, player2_spec),
  }
}

#[cfg(test)]
mod test {
}  // mod test