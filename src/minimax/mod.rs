extern crate time;

use std::fmt;
use std::marker::PhantomData;

use def::Agent;
use def::Evaluator;
use def::State;

pub struct MinimaxReport {
  score: f32
}

impl fmt::Display for MinimaxReport {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    try!(write!(f, "Score: {}", self.score));
    Ok(())
  }
}

pub struct MiniMaxAgent<'a, S: State<'a> + 'a, E: Evaluator<'a, S> + Clone> {
  _s: PhantomData<&'a S>,
  evaluator: E,
  max_depth: i32,
  time_limit: f64,
}

impl<'a, S: State<'a>, E: Evaluator<'a, S> + Clone> MiniMaxAgent<'a, S, E> {
  pub fn new(evaluator: &E, max_depth: i32, time_limit: f64) -> Self {
    assert!(max_depth > 0);
    assert!(time_limit > 0.0);
    MiniMaxAgent {
      _s: PhantomData,
      evaluator: (*evaluator).clone(),
      max_depth: max_depth,
      time_limit: time_limit,
    }
  }

  fn search(&self, state: &S, depth: i32, deadline: f64)
      -> Option<(f32, Option<S::Move>)> {
    if state.is_terminal() || depth == 0 {
      return Some((self.evaluator.evaluate(state), None));
    }

    if time::precise_time_s() >= deadline {
      return None;
    }

    let mut best_move = None;
    let player = state.get_player();
    let mut best_score = if player { -2.0 } else { 2.0 };

    for m in state.iter_moves() {
      let mut state_clone = state.clone();
      state_clone.play(m).unwrap();
      match self.search(&state_clone, depth - 1, deadline) {
        None => return None,
        Some((score, _)) => {
          if player && score > best_score || !player && score < best_score {
            best_score = score;
            best_move = Some(m);
          }
        }
      }
    }

    Some((best_score * 0.999, best_move))
  }
}

impl<'a, S: State<'a>, E: Evaluator<'a, S> + Clone> Agent<'a, S>
  for MiniMaxAgent<'a, S, E> {
  type Report = MinimaxReport;

  fn select_move(&mut self, state: &S) -> Result<(S::Move, MinimaxReport), &'static str> {
    if state.is_terminal() {
      return Err("Terminal state")
    }

    let deadline = time::precise_time_s() + self.time_limit;
    let mut best_move: Option<S::Move> = None;
    let mut best_score: f32 = 0.0;

    for depth in 1..(self.max_depth + 1) {
      match self.search(state, depth, deadline) {
        None => break,
        Some((score, m)) => {
          best_move = m;
          best_score = score;
        }
      }
    }

    if let Some(m) = best_move {
      Ok((m, MinimaxReport{score: best_score}))
    } else {
      Err("No best move?")
    }
  }
}
