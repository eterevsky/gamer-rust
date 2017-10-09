use std;
use std::fmt;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

use def;
use def::Agent;
use def::Evaluator;
use def::State;

pub struct MinimaxReport<M: fmt::Display> {
  score: f32,
  // Principle variation
  pv: Vec<M>,
  samples: u64,
  start_time: Instant,
  end_time: Instant
}

impl<M: fmt::Display> fmt::Display for MinimaxReport<M> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    writeln!(f, "Score: {}", self.score)?;
    write!(f, "PV:")?;

    for m in self.pv.iter() {
      write!(f, " {}", m)?;
    }

    let duration = self.end_time - self.start_time;
    let duration_sec = duration.as_secs() as f64 +
                       duration.subsec_nanos() as f64 * 1E-9;

    writeln!(f, "\nEvaluated: {} positions in {} seconds, {} p/s",
             self.samples, duration_sec,
             self.samples as f64 / duration_sec)?;

    Ok(())
  }
}

impl<M: fmt::Display + Copy> def::AgentReport<M> for MinimaxReport<M> {
  fn get_move(&self) -> M {
    self.pv[0]
  }
}

pub struct MiniMaxAgent<'g, S: State<'g>, E: Evaluator<'g, S>> {
  _s: PhantomData<S>,
  _l: PhantomData<&'g ()>,
  evaluator: E,
  max_depth: i32,
  time_limit: Duration
}

impl<'g, S: State<'g>, E: Evaluator<'g, S>> MiniMaxAgent<'g, S, E> {
  pub fn new(evaluator: E, max_depth: i32, time_limit: Duration) -> Self {
    assert!(max_depth > 0);
    MiniMaxAgent {
      _s: PhantomData,
      _l: PhantomData,
      evaluator: evaluator,
      max_depth: max_depth,
      time_limit: time_limit
    }
  }

  fn search(&self, state: &S, depth: i32, deadline: Instant, lo: f32, hi: f32)
      -> Option<(f32, Vec<S::Move>, u64)> {
    if Instant::now() >= deadline {
      return None;
    }

    if state.is_terminal() || depth == 0 {
      return Some((self.evaluator.evaluate(state), Vec::new(), 1));
    }

    let player = state.get_player();
    let mut best_pv = Vec::new();
    let mut lo = lo / 0.999;
    let mut hi = hi / 0.999;
    let mut samples = 0;

    let mut state_clone = state.clone();

    for m in state.iter_moves() {
      state_clone.play(m).unwrap();
      match self.search(&state_clone, depth - 1, deadline, lo, hi) {
        None => return None,
        Some((score, pv, branch_samples)) => {
          samples += branch_samples;
          if player && score > lo {
            best_pv = pv;
            best_pv.push(m);
            lo = score;
            if score >= hi {
              return Some((score * 0.999, best_pv, samples))
            }
          } else if !player && score < hi {
            best_pv = pv;
            best_pv.push(m);
            hi = score;
            if score <= lo {
              return Some((score * 0.999, best_pv, samples))
            }
          }
        }
      }
      state_clone.undo(m).unwrap();
    }

    let best_score = if player { lo } else { hi };

    Some((best_score * 0.999, best_pv, samples))
  }
}

impl<'a, S: State<'a>, E: Evaluator<'a, S>> Agent<'a, S>
    for MiniMaxAgent<'a, S, E> {
  type Report = MinimaxReport<S::Move>;

  fn select_move(&mut self, state: &S) -> Result<Self::Report, &'static str> {
    if state.is_terminal() {
      return Err("Terminal state");
    }

    let start_time = Instant::now();
    let deadline = start_time + self.time_limit;
    let mut best_pv = Vec::new();
    let mut best_score: f32 = 0.0;
    let mut samples = 0;

    for depth in 1..(self.max_depth + 1) {
      match self.search(state, depth, deadline, std::f32::MIN, std::f32::MAX) {
        None => break,
        Some((score, pv, search_samples)) => {
          samples += search_samples;
          best_pv = pv;
          best_score = score;
        }
      }
    }

    best_pv.reverse();

    if best_pv.is_empty() {
      Err("No best move?")
    } else {
      Ok(
        MinimaxReport{
          score: best_score,
          pv: best_pv,
          samples: samples,
          start_time: start_time,
          end_time: Instant::now()
        })
    }
  }
}

#[cfg(test)]
mod test {

use std::time::Duration;

use def::Agent;
use def::AgentReport;
use def::Game;
use subtractor::{Subtractor};
use terminal_evaluator::TerminalEvaluator;
use super::*;

#[test]
fn subtractor() {
  let mut agent = MiniMaxAgent::new(
      TerminalEvaluator::new(), 10, Duration::from_secs(1));
  let game = Subtractor::new(10, 4);
  let mut state = game.new_game();

  assert_eq!(2, agent.select_move(&state).unwrap().get_move());

  state.play(3).unwrap();

  assert_eq!(3, agent.select_move(&state).unwrap().get_move());
}

}