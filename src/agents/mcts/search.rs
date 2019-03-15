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

    let mut root_node = Node::new(None);

    for _ in 0..max_samples {
      self.sample(self.root_state.clone(), root_node, self.policy);
      if Instant::now() > deadline { break }
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

  fn sample<S: State, P: Policy<S>>(&self, state: S, node: &mut Node<S>) -> f32 {
    if node.children.is_empty() && node.samples > 0 {
      node.expand(&state, self.policy)
    }

    if node.samples == 0 {
      
    }
  }

}

struct Node<S: State> {
  samples: u32,
  score: f32,
  last_move: Option<S::Move>,
  children: Vec<(Self, f32)>,
}

impl<S: State> Node<S> {
  fn new(m: Option<S::Move>) -> Self {
    Node {
      samples: 0,
      score: 0.0,
      last_move: m,
      children: Vec::new()
    }
  }

  fn expand<P: Policy<S>>(&mut self, state: &S, policy: &P) {
    let rated_moves = policy.get_moves(state);
    self.children.reserve_exact(rated_moves.len());
    for m in moves.iter() {
      self.children.push(Node::new(Some(*m)));
    }            
  }
}