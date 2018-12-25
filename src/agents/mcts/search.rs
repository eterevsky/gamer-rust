use rand;
use std::time::Instant;

use crate::def::{Evaluator, Policy, State};
use super::report::MctsReport;

pub struct MctsSearch<S: State, P: Policy<S>, E: Evaluator<S>> {
  policy: P,
  evaluator: E,
  root_state: S,
}

impl<S: State, P: Policy<S>, E: Evaluator<S>> MctsSearch<S, P, E> {
  pub fn new(policy: P, evaluator: E, state: S) -> Self {
    MctsSearch {
      policy,
      evaluator,
      root_state: state,
    }
  }

  pub fn search(
    &self,
    max_samples: u64,
    deadline: Instant,
  ) -> Result<MctsReport<S::Move>, &'static str> {
    if self.root_state.is_terminal() {
      return Err("the state is terminal");
    }

    Ok(MctsReport::new(
      self
        .root_state
        .get_random_move(&mut rand::thread_rng())
        .unwrap(),
      0,
      0.,
      self.root_state.player(),
    ))
  }
}

struct Node<S: State> {
  samples: u32,
  score: f32,
  last_move: S::Move,
  children: Vec<Node<S>>
}

impl<S: State> Node<S> {
  fn expand<P: Policy<S>>(&mut self, policy: &P) {
    
  }
}