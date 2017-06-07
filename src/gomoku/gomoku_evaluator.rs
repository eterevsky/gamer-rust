use def::Evaluator;
use def::State;
use gomoku::gomoku::GomokuState;

#[derive(Clone, Copy, Debug)]
pub struct GomokuEvaluator {}

impl GomokuEvaluator {
  pub fn new() -> GomokuEvaluator {
    GomokuEvaluator {}
  }
}

impl Evaluator<GomokuState> for GomokuEvaluator {
  fn evaluate(&self, state: &GomokuState) -> f32 {
    if state.is_terminal() {
      state.get_payoff().unwrap()
    } else {
      0.0
    }
  }
}
