use def::{Policy, State};

/// A policy that returns all possible moves with equal probability.
pub struct EqualPolicy {}

impl EqualPolicy {
  pub fn new() -> EqualPolicy {
    EqualPolicy {}
  }
}

impl<S: State> Policy<S> for EqualPolicy {
  fn get_moves(&self, state: &S) -> Vec<(S::Move, f32)> {
    let moves: Vec<S::Move> = state.iter_moves().collect();
    let move_weight = 1. / moves.len() as f32;
    moves.iter().map(|m| (m.clone(), move_weight)).collect()
  }
}

#[cfg(test)]
mod test {

  use super::*;
  use def::Game;
  use games::Subtractor;

  #[test]
  fn gen_moves() {
    let game = Subtractor::default(21, 4);
    let state = game.new_game();
    let policy = EqualPolicy::new();
    let weighted_moves = policy.get_moves(&state);

    assert_eq!(3, weighted_moves.len());
    assert_eq!(1./3., weighted_moves[0].1);
    assert_eq!(1./3., weighted_moves[1].1);
    assert_eq!(1./3., weighted_moves[2].1);
  }

}  // mod test