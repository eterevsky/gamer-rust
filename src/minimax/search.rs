use rand::{Rng, XorShiftRng, weak_rng};
use std;
use std::marker::PhantomData;
use std::time::Instant;

use def::{Evaluator, State};
use minimax::MinimaxReport;

struct MinimaxSearch<'e, 'g, S: State<'g>, E: Evaluator<'g, S> + 'e> {
  _s: PhantomData<S>,
  _l: PhantomData<&'g ()>,
  evaluator: &'e E,
  deadline: Option<Instant>,
  // Discount per depth.
  discount: Vec<f32>,
  max_depth: u32,

  depth: u32,
  leaves: u64,
  rng: XorShiftRng
}

enum SearchResult<'g, S: State<'g>> {
  Deadline,  // Deadline exceeded while scanning the branch.
  Lower,
  Higher,
  Found(f32, Vec<S::Move>)
}

impl<'e, 'g, S: State<'g>, E: Evaluator<'g, S> + 'e> MinimaxSearch<'e, 'g, S, E> {
  pub fn new(evaluator: &'e E, depth: u32, discount: f32) -> Self {
    assert!(discount <= 1.0);
    let discount_vec = (0..(depth + 1)).map(|d| discount.powi(d as i32)).collect();
    MinimaxSearch {
      _l: PhantomData,
      _s: PhantomData,

      evaluator,
      deadline: None,
      discount: discount_vec,
      max_depth: depth,

      depth: 0,
      leaves: 0,
      rng: weak_rng()
    }
  }

  fn search(&mut self, state: &S, lo: f32, hi: f32) -> SearchResult<'g, S> {
    assert!(lo <= hi);
    if self.deadline.is_some() && Instant::now() >= self.deadline.unwrap() {
      return SearchResult::Deadline;
    }

    if state.is_terminal() || self.depth == self.max_depth {
      self.leaves += 1;
      let evaluation = self.discount[self.depth as usize] * self.evaluator.evaluate(state);
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
    let mut result = if player {SearchResult::Lower} else {SearchResult::Higher};

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
        },
        SearchResult::Lower => {
          if !player {
            result = SearchResult::Lower;
            break;
          }
        },
        SearchResult::Higher => {
          if player {
            result = SearchResult::Higher;
            break;
          }
        },
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
    };

    self.depth -= 1;

    result
  }
}

pub fn minimax_fixed_depth<'g, S, E>(
    state: &S, evaluator: &E, depth: u32, discount: f32)
    -> MinimaxReport<S::Move>
    where S: State<'g>, E: Evaluator<'g, S>{
  let mut minimax = MinimaxSearch::new(evaluator, depth, discount);
  let start_time = Instant::now();
  if let SearchResult::Found(score, pv) =
      minimax.search(state, std::f32::MIN, std::f32::MAX) {
    let mut pv = pv;
    pv.reverse();

    MinimaxReport {
      score,
      pv,
      samples: minimax.leaves,
      duration: start_time.elapsed(),
      player: state.get_player()
    }
  } else {
    panic!()
  }
}


#[cfg(test)]
mod test {

use def::{AgentReport, Game};
use subtractor::Subtractor;
use terminal_evaluator::TerminalEvaluator;
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

}