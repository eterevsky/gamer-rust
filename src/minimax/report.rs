use std::fmt;
use std::time::Duration;

use def::AgentReport;

#[derive(Clone)]
pub struct MinimaxReport<M: fmt::Display + 'static> {
  pub score: f32,
  // Principle variation
  pub pv: Vec<M>,
  pub samples: u64,
  pub duration: Duration,
  pub player: bool
}

impl<M: fmt::Display + 'static> fmt::Display for MinimaxReport<M> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "Player {}:", if self.player { 1 } else { 2 })?;

    for m in self.pv.iter() {
      write!(f, " {}", m)?;
    }

    writeln!(f, ", score {}, evaluated {} positions",
             self.score, self.samples)?;

    Ok(())
  }
}

impl<M: fmt::Display + Copy + 'static> AgentReport<M> for MinimaxReport<M> {
  fn get_move(&self) -> M {
    self.pv[0]
  }
}

