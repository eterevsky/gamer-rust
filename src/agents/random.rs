//! Implementation of a trivial agent, selecting random valid moves.

use rand::FromEntropy;
use rand::rngs::SmallRng;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;

use crate::def::{Agent, AgentReport, State};
use crate::spec::AgentSpec;

#[derive(Debug)]
pub struct RandomAgentReport<M> {
  m: M,
  player: bool
}

impl<M: Copy + Display + 'static> AgentReport<M> for RandomAgentReport<M> {
  fn get_move(&self) -> M {
    self.m
  }
}

impl<M: Display> Display for RandomAgentReport<M> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "Player {} random move: {}", self.player, self.m)

  }
}

pub struct RandomAgent {
  rng: RefCell<SmallRng>
}

impl RandomAgent {
  pub fn new() -> Self {
    RandomAgent { rng: RefCell::new(SmallRng::from_entropy()) }
  }
}

impl<S: State> Agent<S> for RandomAgent {
  fn select_move(&self, state: &S)
      -> Result<Box<AgentReport<S::Move>>, &'static str> {
    match state.get_random_move(&mut *self.rng.borrow_mut()) {
      Some(m) => Ok(Box::new(RandomAgentReport{m, player: state.player()})),
      None => Err("Terminal position")
    }
  }

  fn spec(&self) -> AgentSpec {
    AgentSpec::Random
  }
}
