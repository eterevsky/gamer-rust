use rand::{Rng, weak_rng};
use std::clone::Clone;
use std::fmt;

use def::{Game, State};
use gomoku::Gomoku;
use hexapawn::Hexapawn;
use registry::create_agent;
use spec::{AgentSpec, GameSpec};
use subtractor::Subtractor;

struct Participant {
  id: usize,
  agent_spec: AgentSpec
}

#[derive(Clone, Copy, Debug)]
struct GameResult {
  player1_id: usize,
  player2_id: usize,
  payoff: f32
}

struct Worker<G: Game> {
  game: &'static G
}

impl<G: Game> Worker<G> {
  fn new(game: &'static G) -> Self {
    Worker {
      game
    }
  }

  fn run_game(&mut self, player1: &Participant, player2: &Participant)
      -> GameResult {
    let agent1 = create_agent(self.game, &player1.agent_spec);
    let agent2 = create_agent(self.game, &player2.agent_spec);
    let mut state = self.game.new_game();
    while !state.is_terminal() {
      let report = if state.get_player() {
        agent1.select_move(&state)
      } else {
        agent2.select_move(&state)
      };
      let m = report.unwrap().get_move();
      state.play(m).unwrap();
    }

    GameResult {
      player1_id: player1.id,
      player2_id: player2.id,
      payoff: state.get_payoff().unwrap()
    }
  }
}

struct Ratings {
  ratings: Vec<f32>,
  played_games: Vec<f32>
}

impl Ratings {
  fn new() -> Self {
    Ratings {
      ratings: vec![],
      played_games: vec![]
    }
  }

  fn add_game(&mut self, player1_id: usize, player2_id: usize, payoff: f32) {
    while self.ratings.len() <= player1_id || self.ratings.len() <= player2_id {
      self.ratings.push(0.0);
      self.played_games.push(0.0);
    }

    self.played_games[player1_id] += 1.0;
    self.played_games[player2_id] += 1.0;

    let rating_diff = self.ratings[player1_id] - self.ratings[player2_id];
    let expected_payoff = (rating_diff / 400.0).tanh();
    let payoff_err = payoff - expected_payoff;

    self.ratings[player1_id] +=
        400.0 / self.played_games[player1_id] * payoff_err;
    self.ratings[player2_id] -=
        400.0 / self.played_games[player2_id] * payoff_err;
  }
}

impl fmt::Display for Ratings {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let mut indices: Vec<_> = (0..self.ratings.len()).collect();
    indices.sort_unstable_by(
        |&i, &j| self.ratings[j].partial_cmp(&self.ratings[i]).unwrap());
    for i in indices {
      writeln!(f, "{}  {:.1}  {}", i, self.ratings[i], self.played_games[i])?;
    }
    Ok(())
  }
}

pub trait ILadder {
  fn add_participant(&mut self, agent_spec: &AgentSpec);
  fn run_full_round(&mut self);
}

pub struct Ladder<G: Game> {
  participants: Vec<Participant>,
  results: Vec<GameResult>,
  workers: Vec<Worker<G>>,
  ratings: Ratings
}

impl<G: Game> Ladder<G> {
  pub fn new(game: &'static G) -> Self {
    Ladder {
      // game,
      participants: Vec::new(),
      results: Vec::new(),
      workers: vec![Worker::new(game)],
      ratings: Ratings::new()
    }
  }
}

impl<G: Game> ILadder for Ladder<G> {
  fn add_participant(&mut self, agent_spec: &AgentSpec) {
    let id = self.participants.len();
    self.participants.push(
        Participant { id, agent_spec: (*agent_spec).clone() })
  }

  fn run_full_round(&mut self) {
    let mut pairs = vec![];
    for i in 0..self.participants.len() {
      for j in 0..self.participants.len() {
        if i != j {
          pairs.push((i, j));
        }
      }
    }

    weak_rng().shuffle(&mut pairs);

    for (i, j) in pairs {
      let player1 = &self.participants[i];
      let player2 = &self.participants[j];
      let result = self.workers[0].run_game(player1, player2);
      self.results.push(result);
      println!("{:?}", result);
      self.ratings.add_game(player1.id, player2.id, result.payoff);
      println!("{}", self.ratings);
    }
  }
}

pub fn ladder_for_game(game_spec: &GameSpec) -> Box<ILadder> {
  match game_spec {
    &GameSpec::Gomoku => Box::new(Ladder::new(Gomoku::default())),
    &GameSpec::Hexapawn(width, height) =>
        Box::new(Ladder::new(Hexapawn::default(width, height))),
    &GameSpec::Subtractor(start, max_sub) =>
        Box::new(Ladder::new(Subtractor::default(start, max_sub)))
  }
}

#[cfg(test)]
mod test {

use super::*;
use subtractor::Subtractor;
use spec::{AgentSpec, EvaluatorSpec};

#[test]
fn run_game() {
  let game = Subtractor::default(21, 4);
  let participant1 = Participant {
    id: 0,
    agent_spec: AgentSpec::Random,
  };
  let participant2 = Participant {
    id: 1,
    agent_spec: AgentSpec::Minimax {
      depth: 3,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::Terminal
    },
  };
  let mut worker = Worker::new(game);
  let result = worker.run_game(&participant1, &participant2);
  assert_eq!(0, result.player1_id);
  assert_eq!(1, result.player2_id);
  assert_eq!(-1.0, result.payoff);
}

}  // mod tests
