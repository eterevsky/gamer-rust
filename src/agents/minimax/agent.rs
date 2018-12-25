use rand;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

use crate::def::{Agent, AgentReport, Evaluator, State};
use crate::spec::AgentSpec;
use super::search::{MinimaxSearch, SearchResult};
use super::report::MinimaxReport;

pub struct MinimaxAgent<S: State, E: Evaluator<S>> {
  _state: PhantomData<S>,
  evaluator: E,
  max_depth: u32,
  time_limit: Option<Duration>,
}

impl<S: State, E: Evaluator<S>> MinimaxAgent<S, E> {
  pub fn new(
    evaluator: E,
    max_depth: u32,
    time_limit: Option<Duration>,
  ) -> Self {
    assert!(max_depth > 0);
    MinimaxAgent {
      _state: PhantomData{},
      evaluator,
      max_depth,
      time_limit,
    }
  }
}

impl<S: State, E: Evaluator<S>> Agent<S> for MinimaxAgent<S, E> {
  fn select_move(
    &self,
    state: &S,
  ) -> Result<Box<AgentReport<S::Move>>, &'static str> {
    if state.is_terminal() {
      return Err("Terminal state");
    }

    let start_time = Instant::now();
    let deadline = match self.time_limit {
      Some(d) => Some(start_time + d),
      None => None,
    };

    let mut minimax = MinimaxSearch::new(&self.evaluator, 1, 0.999, deadline);
    let mut report = MinimaxReport {
      score: 0.0,
      pv: vec![state.get_random_move(&mut rand::thread_rng()).unwrap()],
      samples: 0,
      duration: Duration::new(0, 0),
      player: state.player(),
      depth: 0,
    };

    for depth in 1..(self.max_depth + 1) {
      minimax.set_depth(depth);
      let result = minimax.full_search(state);
      match result {
        SearchResult::Deadline => break,
        SearchResult::Found(score, mut pv) => {
          pv.reverse();
          report.score = score;
          report.pv = pv;
          report.samples = minimax.leaves;
          report.depth = depth;
        }
        _ => unreachable!(),
      }
    }

    report.duration = Instant::now() - start_time;

    assert!(!report.pv.len() > 0);

    Ok(Box::new(report))
  }

  fn spec(&self) -> AgentSpec {
    AgentSpec::Minimax {
      depth: self.max_depth,
      time_per_move: convert_duration(self.time_limit),
      evaluator: self.evaluator.spec(),
      name: String::new()
    }
  }
}

fn convert_duration(duration: Option<Duration>) -> f64 {
  match duration {
    None => 0.0,
    Some(d) => d.as_secs() as f64 + d.subsec_nanos() as f64 * 1E-9,
  }
}

#[cfg(test)]
mod test {

  use crate::def::{Agent, Game};
  use crate::games::Subtractor;
  use crate::evaluators::TerminalEvaluator;
  
  use super::*;

  #[test]
  fn subtractor() {
    let agent = MinimaxAgent::new(TerminalEvaluator::new(), 10, None);
    let game = Subtractor::new(10, 4);
    let mut state = game.new_game();

    let report = agent.select_move(&state).unwrap();
    println!("{}", report);
    assert_eq!(2, report.get_move());

    state.play(3).unwrap();

    let report = agent.select_move(&state).unwrap();
    println!("{}", report);
    assert_eq!(3, report.get_move());
  }

}
