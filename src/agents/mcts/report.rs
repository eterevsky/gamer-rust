use std::fmt;

use crate::def::AgentReport;

#[derive(Clone)]
pub struct MctsReport<M: fmt::Display + Clone + 'static> {
  best_move: M,
  // Total number of playouts.
  samples: u64,
  score: f32,
  // Player, that makes the move.
  player: bool,
}

impl<M: fmt::Display + Clone + 'static> MctsReport<M> {
  pub fn new(best_move: M, samples: u64, score: f32, player: bool) -> Self {
    MctsReport {
      best_move,
      samples,
      score,
      player
    }
  }
} 

impl<M: fmt::Display + Clone + 'static> fmt::Display for MctsReport<M> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    writeln!(
      f,
      "Player {}: {}, score {:.3}, playouts {}",
      if self.player { 1 } else { 2 },
      self.best_move,
      self.score,
      self.samples
    )
  }
}

impl<M: fmt::Display + Clone + 'static> AgentReport<M> for MctsReport<M> {
  fn get_move(&self) -> M {
    self.best_move.clone()
  }
}
