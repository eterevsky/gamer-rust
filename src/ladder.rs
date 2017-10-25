use std::clone::Clone;
use std::fmt;

use def::{Agent, Game, State};
use gomoku::Gomoku;
use hexapawn::Hexapawn;
use registry::create_agent;
use spec::{AgentSpec, GameSpec};
use subtractor::Subtractor;

struct Participant<G: Game> {
  game: &'static G,
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

  fn get_agent(&self, _id: usize, spec: &AgentSpec) -> Box<Agent<G::State>> {
    create_agent(self.game, spec)
    
    // if let agent = Some(self.agents.get(id)) {
    //   agent.borrow_mut()
    // } else {
    //   let agent = create_agent(self.game, spec);
    //   self.agents.insert(id, agent);
    //   self.agents.get(id).unwrap().borrow_mut()
    // }
  }

  fn run_game(&mut self, player1: &Participant<G>, player2: &Participant<G>)
      -> GameResult {
    let agent1 = self.get_agent(player1.id, &player1.agent_spec);
    let agent2 = self.get_agent(player2.id, &player2.agent_spec);
    let mut state = self.game.new_game();
    while !state.is_terminal() {
      let m = if state.get_player() {
        agent1.select_move(&state)
      } else {
        agent2.select_move(&state)
      };
      let m = m.unwrap().get_move();
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

    self.ratings[player1_id] += 400.0 / self.played_games[player1_id] * (payoff - expected_payoff);
    self.ratings[player2_id] -= 400.0 / self.played_games[player2_id] * (payoff - expected_payoff);
  }
}

impl fmt::Display for Ratings {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    for i in 0..self.ratings.len() {
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
  game: &'static G,
  participants: Vec<Participant<G>>,
  results: Vec<GameResult>,
  workers: Vec<Worker<G>>,
  ratings: Ratings
}

impl<G: Game> Ladder<G> {
  pub fn new(game: &'static G) -> Self {
    Ladder {
      game,
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
    self.participants.push(Participant {
      game: self.game,
      id,
      agent_spec: (*agent_spec).clone()})
  }

  fn run_full_round(&mut self) {
    for ref player1 in self.participants.iter() {
      for ref player2 in self.participants.iter() {
        if player1.id == player2.id { continue };
        let result = self.workers[0].run_game(player1, player2);
        self.results.push(result);
        println!("{:?}", result);
        self.ratings.add_game(player1.id, player2.id, result.payoff);
        println!("{}", self.ratings);
      }
    }
  }
}

// impl<G: Game> Drop for Ladder<G> {
//   fn drop(&mut self) {
//     for worker in &mut self.workers {
//       if let Some(some_worker) = worker.take() {
//         some_worker.join().unwrap();
//       }
//     }
//   }
// }

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
    game,
    id: 0,
    agent_spec: AgentSpec::Random,
    rating: 0.0
  };
  let participant2 = Participant {
    game,
    id: 1,
    agent_spec: AgentSpec::Minimax {
      depth: 3,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::Terminal
    },
    rating: 0.0
  };
  let mut worker = Worker::new(game);
  let result = worker.run_game(participant1, participant2);
  assert_eq!(0, result.player1_id);
  assert_eq!(1, result.player2_id);
  assert_eq!(-1.0, result.payoff);
}

}  // mod tests
