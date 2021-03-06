use crate::def::{Evaluator, State};
use crate::spec::EvaluatorSpec;

#[derive(Clone, Debug)]
pub struct TerminalEvaluator {}

impl TerminalEvaluator {
  pub fn new() -> TerminalEvaluator {
    TerminalEvaluator {}
  }
}

impl<S: State> Evaluator<S> for TerminalEvaluator {
  fn evaluate(&self, state: &S) -> f32 {
    if state.is_terminal() {
      state.payoff().unwrap()
    } else {
      0.0
    }
  }

  fn spec(&self) -> EvaluatorSpec {
    EvaluatorSpec::Terminal
  }
}

#[cfg(test)]
mod test {

use crate::def::Game;
use crate::def::Evaluator;
use crate::games::{Gomoku, Subtractor};

use super::*;

#[test]
fn subtractor() {
  let game = Subtractor::new(4, 4);
  let mut state = game.new_game();
  let evaluator = TerminalEvaluator::new();

  assert_eq!(0.0, evaluator.evaluate(&state));
  state.play(3).unwrap();
  assert_eq!(0.0, evaluator.evaluate(&state));
  state.play(1).unwrap();
  assert_eq!(-1.0, evaluator.evaluate(&state));
}

#[test]
fn gomoku() {
  let evaluator = TerminalEvaluator::new();
  let game = Gomoku::new();
  let mut state = game.new_game();
  state.play("J10".parse().unwrap()).unwrap();
  assert_eq!(0.0, evaluator.evaluate(&state));
}

}