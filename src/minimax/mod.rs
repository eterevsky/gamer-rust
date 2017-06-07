extern crate time;

use std::marker::PhantomData;

use def::Agent;
use def::Evaluator;
use def::State;

pub struct MiniMaxAgent<S: State, E: Evaluator<S> + Clone> {
  _s: PhantomData<S>,
  evaluator: E,
  max_depth: i32,
  time_limit: f64
}

impl<S: State, E: Evaluator<S> + Clone> MiniMaxAgent<S, E> {
  pub fn new(evaluator: &E, max_depth: i32, time_limit: f64) -> Self {
    assert!(max_depth > 0);
    assert!(time_limit > 0.0);
    MiniMaxAgent {
      _s: PhantomData,
      evaluator: (*evaluator).clone(),
      max_depth: max_depth,
      time_limit: time_limit
    }
  }

  fn search(&self, state: &S, depth: i32, deadline: f64) -> Option<(f32, Option<S::Move>)> {
    if state.is_terminal() {
      return Some((state.get_payoff().unwrap(), None))
    }

    if depth == 0 {
      return Some((self.evaluator.evaluate(state), None))
    }

    if time::precise_time_s() >= deadline {
      return None
    }

    let mut best_move = None;
    let player = state.get_player();
    let mut best_score = if player {-2.0} else {2.0};

    for m in state.iter_moves() {
      let mut state_clone = state.clone();
      state_clone.play(m);
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

impl<S: State, E: Evaluator<S> + Clone> Agent<S> for MiniMaxAgent<S, E> {
  fn select_move(&mut self, state: &S) -> Option<S::Move> {
    if state.is_terminal() {
      return None
    }

    let deadline = time::precise_time_s() + self.time_limit;
    let mut best_move: Option<S::Move> = None;
    for depth in 1..(self.max_depth + 1) {
      match self.search(state, depth, deadline) {
        None => break,
        Some((_, m)) => {best_move = m;}
      }
    }

    best_move
  }
}
