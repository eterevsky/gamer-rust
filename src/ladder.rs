use rand::{Rng, weak_rng};
use std::clone::Clone;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{JoinHandle, spawn};

use def::{Game, State};
use gomoku::Gomoku;
use hexapawn::Hexapawn;
use registry::create_agent;
use spec::{AgentSpec, GameSpec};
use subtractor::Subtractor;

#[derive(Clone)]
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

enum Job {
  Stop,
  Play(Participant, Participant)
}

struct Worker<G: Game> {
  game: &'static G,
  jobs_receiver: Arc<Mutex<Receiver<Job>>>,
  results_sender: Sender<GameResult>
}

impl<G: Game> Worker<G> {
  fn new(
      game: &'static G,
      jobs_receiver: Arc<Mutex<Receiver<Job>>>,
      results_sender: Sender<GameResult>) -> Self {
    Worker {
      game,
      jobs_receiver,
      results_sender
    }
  }

  fn run_game(&self, player1: &Participant, player2: &Participant)
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

  fn run(&mut self) {
    loop {
      let job = self.jobs_receiver.lock().unwrap().recv().unwrap();
      match job {
        Job::Stop => break,
        Job::Play(player1, player2) => {
          let result = self.run_game(&player1, &player2);
          self.results_sender.send(result).unwrap();
        }
      }
    }
  }
}

fn start_worker<G: Game>(
    game: &'static G,
    jobs_receiver: Arc<Mutex<Receiver<Job>>>,
    results_sender: Sender<GameResult>) -> JoinHandle<()> {
  spawn(move || {
    let mut worker = Worker::new(game, jobs_receiver, results_sender);
    worker.run()
  })
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
        400.0 / self.played_games[player1_id].sqrt() * payoff_err;
    self.ratings[player2_id] -=
        400.0 / self.played_games[player2_id].sqrt() * payoff_err;
  }
}

impl fmt::Display for Ratings {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let mut indices: Vec<_> = (0..self.ratings.len()).collect();
    indices.sort_unstable_by(
        |&i, &j| self.ratings[j].partial_cmp(&self.ratings[i]).unwrap());
    for i in indices {
      writeln!(f, "{}  {:.1}  {}", i, self.ratings[i] - self.ratings[0],
               self.played_games[i])?;
    }
    Ok(())
  }
}

pub trait ILadder {
  fn add_participant(&mut self, agent_spec: &AgentSpec);
  fn run_full_round(&mut self);
}

pub struct Ladder {
  participants: Vec<Participant>,
  results: Vec<GameResult>,
  threads: Vec<JoinHandle<()>>,
  ratings: Ratings,
  jobs_sender: Sender<Job>,
  results_receiver: Receiver<GameResult>
}

impl Ladder {
  pub fn new<G: Game>(game: &'static G, nthreads: u32) -> Self {
    let (jobs_sender, jobs_receiver) = channel();
    let jobs_receiver = Arc::new(Mutex::new(jobs_receiver));
    let (results_sender, results_receiver) = channel();

    let mut threads = vec![];
    for _ in 0..nthreads {
      threads.push(
          start_worker(game, jobs_receiver.clone(), results_sender.clone()));
    }

    Ladder {
      participants: Vec::new(),
      results: Vec::new(),
      ratings: Ratings::new(),
      threads,
      jobs_sender,
      results_receiver
    }
  }

  pub fn add_participant(&mut self, agent_spec: &AgentSpec) {
    let id = self.participants.len();
    self.participants.push(
        Participant { id, agent_spec: (*agent_spec).clone() })
  }

  pub fn run_full_round(&mut self, nrounds: u32) {
    let mut pairs = vec![];
    for i in 0..self.participants.len() {
      for j in 0..self.participants.len() {
        if i != j {
          pairs.push((i, j));
        }
      }
    }

    let mut rng = weak_rng();
    for _ in 0..nrounds {
      rng.shuffle(&mut pairs);

      for &(i, j) in pairs.iter() {
        let job = Job::Play(self.participants[i].clone(),
                            self.participants[j].clone());
        self.jobs_sender.send(job).unwrap();
      }
    }

    for _ in 0..pairs.len() * nrounds as usize {
      let result = self.results_receiver.recv().unwrap();
      self.results.push(result);
      println!("{:?}", result);
      self.ratings.add_game(
          result.player1_id, result.player2_id, result.payoff);
      println!("{}", self.ratings);
    }
  }
}

impl Drop for Ladder {
  fn drop(&mut self) {
    for _ in 0..self.threads.len() {
      self.jobs_sender.send(Job::Stop).unwrap();
    }
    while !self.threads.is_empty() {
      let thread = self.threads.pop().unwrap();
      thread.join().unwrap();
    }
  }
}

pub fn ladder_for_game(game_spec: &GameSpec) -> Ladder {
  match game_spec {
    &GameSpec::Gomoku => Ladder::new(Gomoku::default(), 8),
    &GameSpec::Hexapawn(width, height) =>
        Ladder::new(Hexapawn::default(width, height), 8),
    &GameSpec::Subtractor(start, max_sub) =>
        Ladder::new(Subtractor::default(start, max_sub), 8)
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
