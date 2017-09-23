//! Implementation of a trivial agent, selecting random valid moves.

use rand;

use gomoku::def;

pub struct RandomAgent {
  rng: rand::XorShiftRng;
}

impl RandomAgent {
  pub fn new() -> Self {
    RandomAgent { rng: rand.weak_rng() }
  }
}

impl<'g, S: def::State<'a>> def::Agent<'g, S> for RandomAgent {
  type Report = &'static str;

  fn select_move(&mut self, state: &S)
      -> Result<(S::Move, &'static str), &'static str> {
    match state.get_random_move(&mut self.rng) {
      Some(m) => Ok(m, "Random move"),
      None => Error("Terminal position"),
    }
  }
}
