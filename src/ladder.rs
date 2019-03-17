use rand::{Rng, FromEntropy};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use std::clone::Clone;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

use crate::def::{Game, State};
use crate::ratings::Ratings;
use crate::registry::create_agent;
use crate::spec::AgentSpec;

#[derive(Clone)]
struct Participant {
  id: usize,
  agent_spec: AgentSpec,
}

#[derive(Clone, Copy, Debug)]
pub struct GameResult {
  pub player1_id: usize,
  pub player2_id: usize,
  pub payoff: f32,
}

enum Job {
  Stop,
  Play(Participant, Participant),
}

pub fn play_game<G: Game>(
  game: &'static G,
  player1: &AgentSpec,
  player2: &AgentSpec,
  output: bool,
) -> f32 {
  let agent1 = create_agent(game, player1);
  let agent2 = create_agent(game, player2);
  let mut state = game.new_game();
  while !state.is_terminal() {
    if output {
      println!("{}", state);
    }
    let report = if state.player() {
      agent1.select_move(&state)
    } else {
      agent2.select_move(&state)
    };
    let report = report.unwrap();
    if output {
      println!("{}", report);
    }
    state.play(report.get_move()).unwrap();
  }
  let payoff = state.payoff().unwrap();
  if output {
    println!("{}\nPayoff: {}", state, payoff);
  }
  payoff
}

struct Worker<G: Game> {
  game: &'static G,
  jobs_receiver: Arc<Mutex<Receiver<Job>>>,
  results_sender: Sender<GameResult>,
}

impl<G: Game> Worker<G> {
  fn new(
    game: &'static G,
    jobs_receiver: Arc<Mutex<Receiver<Job>>>,
    results_sender: Sender<GameResult>,
  ) -> Self {
    Worker {
      game,
      jobs_receiver,
      results_sender,
    }
  }

  fn run_game(
    &self,
    player1: &Participant,
    player2: &Participant,
  ) -> GameResult {
    let payoff = play_game(
      self.game,
      &player1.agent_spec,
      &player2.agent_spec,
      false,
    );

    GameResult {
      player1_id: player1.id,
      player2_id: player2.id,
      payoff,
    }
  }

  fn run(&mut self) {
    loop {
      let job = self
        .jobs_receiver
        .lock()
        .unwrap()
        .recv()
        .unwrap();
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
  results_sender: Sender<GameResult>,
) -> JoinHandle<()> {
  spawn(move || {
    let mut worker = Worker::new(game, jobs_receiver, results_sender);
    worker.run()
  })
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
  results_receiver: Receiver<GameResult>,
}

impl Ladder {
  pub fn new<G: Game>(game: &'static G, nthreads: usize) -> Self {
    let (jobs_sender, jobs_receiver) = channel();
    let jobs_receiver = Arc::new(Mutex::new(jobs_receiver));
    let (results_sender, results_receiver) = channel();

    let mut threads = vec![];
    for _ in 0..nthreads {
      threads.push(start_worker(
        game,
        jobs_receiver.clone(),
        results_sender.clone(),
      ));
    }

    Ladder {
      participants: Vec::new(),
      results: Vec::new(),
      ratings: Ratings::new(true),
      threads,
      jobs_sender,
      results_receiver,
    }
  }

  /// Adds a new agent to the ladder and returns an id that it was assigned.
  pub fn add_participant(&mut self, agent_spec: &AgentSpec) -> usize {
    let id = self.participants.len();
    self.participants.push(Participant {
      id,
      agent_spec: agent_spec.clone(),
    });
    id
  }

  pub fn get_participant<'a>(&'a self, id: usize) -> &'a AgentSpec {
    &self.participants[id].agent_spec
  }

  /// Adds a participant and estimate its rating.
  ///
  /// Adds a new agent to the ladded and runs `ngames` games with existing
  /// participants to estimate its rating. The ratings of existing participants
  /// will be updates as well.
  pub fn add_participant_and_rank(
    &mut self,
    agent_spec: &AgentSpec,
    ngames: usize,
  ) -> (usize, f32) {
    let prior_participants = self.participants.len();
    let id = self.add_participant(agent_spec);
    let mut rng = SmallRng::from_entropy();

    for _ in 0..ngames {
      let opponent = rng.gen_range(0, prior_participants);
      let (player1, player2) = if rng.gen_range(0, 2) > 0 {
        (id, opponent)
      } else {
        (opponent, id)
      };
      self
        .jobs_sender
        .send(Job::Play(
          self.participants[player1].clone(),
          self.participants[player2].clone(),
        ))
        .unwrap();
    }

    for _ in 0..ngames {
      let result = self.results_receiver.recv().unwrap();
      self.ratings.add_game(result);
    }
    self.ratings.full_update();      

    (id, self.ratings.get_rating(id))
  }

  pub fn get_rating(&self, id: usize) -> f32 {
    self.ratings.get_rating(id)
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

    let mut rng = SmallRng::from_entropy();
    for _ in 0..nrounds {
      pairs.shuffle(&mut rng);

      for &(i, j) in pairs.iter() {
        let job = Job::Play(
          self.participants[i].clone(),
          self.participants[j].clone(),
        );
        self.jobs_sender.send(job).unwrap();
      }
    }

    for _ in 0..pairs.len() * nrounds as usize {
      let result = self.results_receiver.recv().unwrap();
      self.results.push(result);
      println!(
        "{} v {} {}\n",
        self.print_participant(result.player1_id),
        self.print_participant(result.player2_id),
        result.payoff
      );
      self.ratings.add_game(result);
      self.ratings.full_update();
      self.print_all();
    }
  }

  pub fn print_all(&self) {
    println!(
      "{}",
      self.ratings.print(
        (0..self.participants.len())
          .map(|i| self.print_participant(i))
          .collect::<Vec<&str>>()
      )
    );
  }

  fn print_participant<'a>(&'a self, id: usize) -> &'a str {
    match self.participants[id].agent_spec {
      AgentSpec::Random => "random",
      AgentSpec::Human => "human",
      AgentSpec::Minimax {
        depth: _,
        time_per_move: _,
        evaluator: _,
        ref name,
      } => name,
      AgentSpec::Mcts {
        samples: _,
        time_per_move: _,
        evaluator: _,
        policy: _,
        ref name,
      } => name,
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

#[cfg(test)]
mod test {
  use std::sync::mpsc::channel;

  use crate::games::{Hexapawn, Subtractor};
  use crate::spec::{AgentSpec, EvaluatorSpec};

  use super::*;

  #[test]
  fn play_hexapawn() {
    let game = Hexapawn::default(3, 3);
    let agent1_spec = AgentSpec::Minimax {
      depth: 3,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::Terminal,
      name: "1".to_string(),
    };
    let agent2_spec = AgentSpec::Minimax {
      depth: 10,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::Terminal,
      name: "2".to_string(),
    };
    assert_eq!(
      -1.0,
      play_game(game, &agent1_spec, &agent2_spec, false)
    );
  }

  #[test]
  fn subtractor_worker() {
    let game = Subtractor::default(21, 4);
    let (jobs_sender, jobs_receiver) = channel();
    let jobs_receiver = Arc::new(Mutex::new(jobs_receiver));
    let (results_sender, results_receiver) = channel();

    let participant1 = Participant {
      id: 0,
      agent_spec: AgentSpec::Random,
    };
    let participant2 = Participant {
      id: 1,
      agent_spec: AgentSpec::Minimax {
        depth: 5,
        time_per_move: 0.0,
        evaluator: EvaluatorSpec::Terminal,
        name: "2".to_string(),
      },
    };

    let thread = start_worker(game, jobs_receiver, results_sender);

    jobs_sender
      .send(Job::Play(participant1, participant2))
      .unwrap();
    let result = results_receiver.recv().unwrap();

    assert_eq!(0, result.player1_id);
    assert_eq!(1, result.player2_id);
    assert_eq!(-1.0, result.payoff);

    jobs_sender.send(Job::Stop).unwrap();
    thread.join().unwrap();
  }

  #[test]
  fn subtractor_ladder() {
    let mut ladder = Ladder::new(Subtractor::default(21, 4), 2);
    let random_id = ladder.add_participant(&AgentSpec::Random);
    let minimax_id = ladder.add_participant(&AgentSpec::Minimax {
      depth: 6,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::Terminal,
      name: "2".to_string(),
    });

    ladder.run_full_round(10);

    assert_eq!(0.0, ladder.get_rating(random_id));
    assert!(ladder.get_rating(minimax_id) > 400.0);
  }

} // mod tests
