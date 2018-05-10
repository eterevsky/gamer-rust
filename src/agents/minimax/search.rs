use rand::{weak_rng, Rng, XorShiftRng};
use std;
use std::f32;
use std::fmt;
use std::time::Instant;

use def::{Evaluator, State};
use super::MinimaxReport;

pub struct MinimaxSearch<'e, S: State + 'e> {
  evaluator: &'e Evaluator<S>,
  deadline: Option<Instant>,
  // Discount per depth.
  discount: Vec<f32>,
  max_depth: u32,

  depth: u32,
  pub leaves: u64,
  rng: XorShiftRng,
}

#[derive(Debug)]
pub enum SearchResult<M: 'static + Copy + fmt::Debug> {
  Deadline, // Deadline exceeded while scanning the branch.
  Lower,
  Higher,
  Found(f32, Vec<M>),
}

impl<'e, S: State> MinimaxSearch<'e, S> {
  pub fn new(
    evaluator: &'e Evaluator<S>,
    depth: u32,
    discount: f32,
    deadline: Option<Instant>,
  ) -> Self {
    assert!(discount <= 1.0);
    let discount_vec = (0..(depth + 1))
      .map(|d| discount.powi(d as i32))
      .collect();
    MinimaxSearch {
      evaluator,
      deadline,
      discount: discount_vec,
      max_depth: depth,

      depth: 0,
      leaves: 0,
      rng: weak_rng(),
    }
  }

  pub fn set_depth(&mut self, depth: u32) {
    assert!(depth > 0);
    let discount = self.discount[1];
    let discount_vec = (0..(depth + 1))
      .map(|d| discount.powi(d as i32))
      .collect();
    self.max_depth = depth;
    self.discount = discount_vec;
  }

  pub fn full_search(&mut self, state: &S) -> SearchResult<S::Move> {
    self.depth = 0;
    self.search(state, f32::MIN, f32::MAX)
  }

  fn search(&mut self, state: &S, lo: f32, hi: f32) -> SearchResult<S::Move> {
    assert!(lo <= hi);
    if self.deadline.is_some() && Instant::now() >= self.deadline.unwrap() {
      return SearchResult::Deadline;
    }

    if state.is_terminal() || self.depth == self.max_depth {
      self.leaves += 1;
      let evaluation =
        self.discount[self.depth as usize] * self.evaluator.evaluate(state);
      if evaluation <= lo {
        return SearchResult::Lower;
      }
      if evaluation >= hi {
        return SearchResult::Higher;
      }
      return SearchResult::Found(evaluation, Vec::new());
    }

    let player = state.get_player();
    let mut lo = lo;
    let mut hi = hi;
    let mut state_clone = state.clone();
    let mut result = if player {
      SearchResult::Lower
    } else {
      SearchResult::Higher
    };

    self.depth += 1;

    let mut moves: Vec<S::Move> = state.iter_moves().collect();
    self.rng.shuffle(&mut moves);
    for m in moves {
      state_clone.play(m).unwrap();
      let child_result = self.search(&state_clone, lo, hi);
      match child_result {
        SearchResult::Deadline => {
          result = SearchResult::Deadline;
          break;
        }
        SearchResult::Lower => {
          if !player {
            result = SearchResult::Lower;
            break;
          }
        }
        SearchResult::Higher => {
          if player {
            result = SearchResult::Higher;
            break;
          }
        }
        SearchResult::Found(score, mut pv) => {
          pv.push(m);
          result = SearchResult::Found(score, pv);
          if player {
            lo = score;
          } else {
            hi = score;
          }
        }
      }
      state_clone.undo(m).unwrap();
    }

    self.depth -= 1;

    result
  }
}

// discount -- a value <= 1.0, but close to it. The payoff will be multiplied
// by if for every move.
pub fn minimax_fixed_depth<S: State, E: Evaluator<S>>(
  state: &S,
  evaluator: &E,
  depth: u32,
  discount: f32,
) -> MinimaxReport<S::Move> {
  let mut minimax = MinimaxSearch::new(evaluator, depth, discount, None);
  let start_time = Instant::now();
  if let SearchResult::Found(score, pv) =
    minimax.search(state, std::f32::MIN, std::f32::MAX)
  {
    let mut pv = pv;
    pv.reverse();

    MinimaxReport {
      score,
      pv,
      samples: minimax.leaves,
      duration: start_time.elapsed(),
      player: state.get_player(),
      depth,
    }
  } else {
    panic!()
  }
}

#[cfg(test)]
mod test {

  use def::{AgentReport, Game};
  use games::Subtractor;
  use evaluators::TerminalEvaluator;
  use super::*;

  #[test]
  fn subtractor() {
    let game = Subtractor::new(5, 4);
    let evaluator = TerminalEvaluator::new();
    let mut state = game.new_game();

    let report = minimax_fixed_depth(&state, &evaluator, 1, 0.5);
    assert_eq!(0.0, report.score);
    assert_eq!(3, report.samples);
    assert!(1 <= report.get_move() && report.get_move() <= 3);

    let report = minimax_fixed_depth(&state, &evaluator, 2, 0.5);
    assert_eq!(0.0, report.score);
    assert!(5 <= report.samples && report.samples <= 8);
    assert_eq!(1, report.get_move());

    let report = minimax_fixed_depth(&state, &evaluator, 3, 0.5);
    assert_eq!(0.125, report.score);
    assert_eq!(1, report.get_move());

    state.play(2).unwrap();
    let report = minimax_fixed_depth(&state, &evaluator, 1, 0.5);
    assert_eq!(-0.5, report.score);
    assert_eq!(3, report.get_move());
  }

  #[test]
  fn subtractor_minimax_10() {
    let game = Subtractor::new(10, 4);
    let mut state = game.new_game();
    let evaluator = TerminalEvaluator::new();

    let mut minimax = MinimaxSearch::new(&evaluator, 10, 0.999, None);
    let result = minimax.full_search(&state);

    match result {
      SearchResult::Found(_, pv) => assert_eq!(2, pv[pv.len() - 1]),
      _ => panic!(),
    };

    state.play(3).unwrap();

    let mut minimax = MinimaxSearch::new(&evaluator, 10, 0.999, None);
    let result = minimax.full_search(&state);

    // println!("{:?}", result);
    match result {
      SearchResult::Found(_, pv) => assert_eq!(3, pv[pv.len() - 1]),
      _ => panic!(),
    };
  }

}
