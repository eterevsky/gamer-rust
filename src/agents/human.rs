use std::fmt;
use std::io;
use std::io::Write;

use def::{Agent, AgentReport, State};
use spec::AgentSpec;

pub struct HumanAgentReport<M: Copy + fmt::Display> {
  m: M
}

impl<M: Copy + fmt::Display> AgentReport<M> for HumanAgentReport<M> {
  fn get_move(&self) -> M {
    self.m
  }
}

impl<M: Copy + fmt::Display> fmt::Display for HumanAgentReport<M> {
  fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    Ok(())
  }
}

pub struct HumanAgent {}

impl<S: State> Agent<S> for HumanAgent {
  fn select_move(&self, state: &S)
      -> Result<Box<AgentReport<S::Move>>, &'static str> {
    loop {
      print!("Player {} move: ", if  state.get_player() { 1 } else { 2 });
      io::stdout().flush().map_err(|_| "Unknown error")?;
      let mut move_str = String::new();
      io::stdin().read_line(&mut move_str)
                 .map_err(|_| "Error while reading user input.")?;
      if let Ok(m) = state.parse_move(move_str.trim()) {
        return Ok(Box::new(HumanAgentReport{m}));
      } else {
        println!("Illegal move.");
      }
    }
  }

  fn spec(&self) -> AgentSpec {
    AgentSpec::Human
  }
}
