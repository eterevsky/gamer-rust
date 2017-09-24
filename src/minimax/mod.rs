extern crate time;

use std;
use std::fmt;
use std::marker::PhantomData;

use def::Agent;
use def::Evaluator;
use def::State;

pub struct MinimaxReport<M: fmt::Display> {
  score: f32,
  // Principle variation
  pv: Vec<M>
}

impl<M: fmt::Display> fmt::Display for MinimaxReport<M> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    writeln!(f, "Score: {}", self.score)?;
    write!(f, "PV:")?;

    for m in self.pv.iter() {
      write!(f, " {}", m)?;
    }

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
      -> Option<(f32, Vec<S::Move>)> {
    if time::precise_time_s() >= deadline {
      return None;
    }

    if state.is_terminal() || depth == 0 {
      return Some((self.evaluator.evaluate(state), Vec::new()));
    }

    let player = state.get_player();
    let mut best_pv = Vec::new();
    let mut best_score = if player { std::f32::MIN } else { std::f32::MAX };

    for m in state.iter_moves() {
      let mut state_clone = state.clone();
      state_clone.play(m).unwrap();
      match self.search(&state_clone, depth - 1, deadline) {
        None => return None,
        Some((score, pv)) => {
          if player && score > best_score || !player && score < best_score {
            best_score = score;
            best_pv = pv;
            best_pv.push(m);
          }
        }
      }
    }

    Some((best_score * 0.999, best_pv))
  }
}

impl<'a, S: State<'a>, E: Evaluator<'a, S> + Clone> Agent<'a, S>
    for MiniMaxAgent<'a, S, E> {
  type Report = MinimaxReport<S::Move>;

  fn select_move(&mut self, state: &S)
      -> Result<(S::Move, Self::Report), &'static str> {
    if state.is_terminal() {
      return Err("Terminal state");
    }

    let deadline = time::precise_time_s() + self.time_limit;
    let mut best_pv = Vec::new();
    let mut best_score: f32 = 0.0;

    for depth in 1..(self.max_depth + 1) {
      match self.search(state, depth, deadline) {
        None => break,
        Some((score, pv)) => {
          best_pv = pv;
          best_score = score;
        }
      }
    }

    best_pv.reverse();

    if best_pv.is_empty() {
      Err("No best move?")
    } else {
      Ok((best_pv[0], MinimaxReport{score: best_score, pv: best_pv}))
    }
  }
}
