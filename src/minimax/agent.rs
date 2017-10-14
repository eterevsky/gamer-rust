use rand;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

use def::{Agent, AgentReport, State, Evaluator};
use minimax::search::{MinimaxSearch, SearchResult};
use minimax::report::MinimaxReport;

pub struct MinimaxAgent<'g, S: State<'g>> {
  _s: PhantomData<S>,
  _l: PhantomData<&'g ()>,
  evaluator: Box<Evaluator<'g, S> + 'g>,
  max_depth: u32,
  time_limit: Option<Duration>
}

impl<'g, S: State<'g>> MinimaxAgent<'g, S> {
  pub fn new_boxed(
      evaluator: Box<Evaluator<'g, S> + 'g>,
      max_depth: u32,
      time_limit: Option<Duration>
  ) -> Self {
    assert!(max_depth > 0);
    MinimaxAgent {
      _s: PhantomData,
      _l: PhantomData,
      evaluator: evaluator,
      max_depth: max_depth,
      time_limit
    }
  }

  pub fn new<E: Evaluator<'g, S> + 'g>(
      evaluator: E,
      max_depth: u32,
      time_limit: Option<Duration>
  ) -> Self {
    Self::new_boxed(Box::new(evaluator), max_depth, time_limit)
  }
}

impl<'g, S: State<'g>> Agent<'g, S> for MinimaxAgent<'g, S> {
  fn select_move(
      &mut self, state: &S
  ) -> Result<Box<AgentReport<S::Move>>, &'static str> {
    if state.is_terminal() {
      return Err("Terminal state");
    }

    let start_time = Instant::now();
    let deadline = match self.time_limit {
      Some(d) => Some(start_time + d),
      None => None
    };

    let mut minimax = MinimaxSearch::new(self.evaluator.borrow(), 1, 0.999, deadline);
    let mut report = MinimaxReport {
      score: 0.0,
      pv: vec![state.get_random_move(&mut rand::weak_rng()).unwrap()],
      samples: 0,
      duration: Duration::new(0, 0),
      player: state.get_player(),
      depth: 0
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
        },
        _ => unreachable!()
      }
    }

    report.duration = Instant::now() - start_time;

    assert!(!report.pv.len() > 0);

    Ok(Box::new(report))
  }
}


#[cfg(test)]
mod test {

use def::{Agent, Game};
use subtractor::Subtractor;
use terminal_evaluator::TerminalEvaluator;
use super::*;

#[test]
fn subtractor() {
  let mut agent = MinimaxAgent::new(TerminalEvaluator::new(), 10, None);
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