use std::clone::Clone;

use def::{Agent, Game, State};
use registry::create_agent;
use spec::AgentSpec;

struct Participant<G: Game> {
  game: &'static G,
  id: u32,
  agent_spec: AgentSpec,
  rating: f32
}

struct GameJob {
  stop: bool,
  player1_id: u32,
  player1_spec: AgentSpec,
  player2_id: u32,
  player2_spec: AgentSpec
}

struct GameResult {
  player1_id: u32,
  player2_id: u32,
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

  fn get_agent(&self, _id: u32, spec: &AgentSpec) -> Box<Agent<G::State>> {
    create_agent(self.game, spec)
    
    // if let agent = Some(self.agents.get(id)) {
    //   agent.borrow_mut()
    // } else {
    //   let agent = create_agent(self.game, spec);
    //   self.agents.insert(id, agent);
    //   self.agents.get(id).unwrap().borrow_mut()
    // }
  }

  fn run_game(&mut self, player1: Participant<G>, player2: Participant<G>)
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

struct Ladder<G: Game> {
  game: &'static G,
  participants: Vec<Participant<G>>,
  results: Vec<GameResult>,
}

impl<G: Game> Ladder<G> {
  pub fn new(game: &'static G) -> Self {
    Ladder {
      game,
      participants: Vec::new(),
      results: Vec::new()
    }
  }

  pub fn add_participant(&mut self, agent_spec: &AgentSpec) {
    let id = self.participants.len();
    self.participants.push(Participant {
      game: self.game,
      id: id as u32,
      agent_spec: (*agent_spec).clone(),
      rating: 0.0 })
  }

  pub fn run_full_round(&self) {
    for ref player1 in self.participants.iter() {
      for ref player2 in self.participants.iter() {
        println!("player1: {:?}", player1.agent_spec);
        println!("player2: {:?}", player2.agent_spec);
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
