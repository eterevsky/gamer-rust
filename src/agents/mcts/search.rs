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

    let mut root_node = Node::new(None, 0.0);

    for _ in 0..max_samples {
      self.sample(self.root_state.clone(), &mut root_node);
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

  fn sample(&self, state: S, node: &mut Node<S>) -> f32 {
    if node.samples == 0 {
      let score = self.evaluator.evaluate(&state);
      node.score = score;
      node.samples = 1;
      return score;
    }

    if state.is_terminal() {
      node.score = state.payoff().unwrap();
      node.samples += 1;
      return node.score;
    }

    if node.children.is_empty() {
      node.expand(&state, &self.policy)
    }

    let mut best_child = None;
    let mut best_value = 0.0;
    let numerator = (node.samples as f32).sqrt();

    for child in node.children.iter_mut() {
      let value = child.score + child.policy_score * numerator / (child.samples + 1) as f32;
      if best_child.is_none() || value > best_value {
        best_child = Some(child);
        best_value = value;
      }
    }

    let mut state = state;
    let best_child = best_child.unwrap();
    state.play(best_child.last_move.unwrap()).unwrap();
    let child_score = self.sample(state, best_child);
    
    node.score = node.score * node.samples as f32 / (node.samples + 1) as f32 +
                 child_score / (node.samples + 1) as f32;
    node.samples += 1;

    return child_score;
  }
}

struct Node<S: State> {
  samples: u32,
  score: f32,
  policy_score: f32,
  last_move: Option<S::Move>,
  children: Vec<Self>,
}

impl<S: State> Node<S> {
  fn new(m: Option<S::Move>, policy_score: f32) -> Self {
    Node {
      samples: 0,
      score: 0.0,
      policy_score,
      last_move: m,
      children: Vec::new()
    }
  }

  fn expand<P: Policy<S>>(&mut self, state: &S, policy: &P) {
    let rated_moves = policy.get_moves(state);
    self.children.reserve_exact(rated_moves.len());
    for (m, w) in rated_moves.iter() {
      self.children.push(Node::new(Some(*m), *w));
    }            
  }
}