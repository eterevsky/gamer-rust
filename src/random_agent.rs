//! Implementation of a trivial agent, selecting random valid moves.

use rand;
use std::fmt;
use std::fmt::Display;

use def::{Agent, AgentReport, State};

#[derive(Debug)]
pub struct RandomAgentReport<M> {
  m: M
}

impl<M: Copy + Display> AgentReport<M> for RandomAgentReport<M> {
  fn get_move(&self) -> M {
    self.m
  }
}

impl<M: Display> Display for RandomAgentReport<M> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}", self.m)?;
    Ok(())
  }
}

pub struct RandomAgent {
  rng: rand::XorShiftRng
}

impl RandomAgent {
  pub fn new() -> Self {
    RandomAgent { rng: rand::weak_rng() }
  }
}

impl<'g, S: State<'g>> Agent<'g, S> for RandomAgent {
  type Report = RandomAgentReport<S::Move>;

  fn select_move(&mut self, state: &S)
      -> Result<RandomAgentReport<S::Move>, &'static str> {
    match state.get_random_move(&mut self.rng) {
      Some(m) => Ok(RandomAgentReport{m}),
      None => Err("Terminal position")
    }
  }
}
