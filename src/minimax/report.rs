use std::fmt;
use std::time::Duration;

use def::AgentReport;

pub struct MinimaxReport<M: fmt::Display> {
  pub score: f32,
  // Principle variation
  pub pv: Vec<M>,
  pub samples: u64,
  pub duration: Duration
}

impl<M: fmt::Display> fmt::Display for MinimaxReport<M> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    writeln!(f, "Score: {}", self.score)?;
    write!(f, "PV:")?;

    for m in self.pv.iter() {
      write!(f, " {}", m)?;
    }

    let duration_sec = self.duration.as_secs() as f64 +
                       self.duration.subsec_nanos() as f64 * 1E-9;

    writeln!(f, "\nEvaluated: {} positions in {} seconds, {} p/s",
             self.samples, duration_sec,
             self.samples as f64 / duration_sec)?;

    Ok(())
  }
}

impl<M: fmt::Display + Copy> AgentReport<M> for MinimaxReport<M> {
  fn get_move(&self) -> M {
    self.pv[0]
  }
}

