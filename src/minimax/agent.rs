use std::marker::PhantomData;
use std::time::Duration;

use def::{Agent, AgentReport, State, Evaluator};
use minimax::search::minimax_fixed_depth;

pub struct MinimaxAgent<'g, S: State<'g>, E: Evaluator<'g, S>> {
  _s: PhantomData<S>,
  _l: PhantomData<&'g ()>,
  evaluator: E,
  max_depth: u32,
}

impl<'g, S: State<'g>, E: Evaluator<'g, S>> MinimaxAgent<'g, S, E> {
  pub fn new(evaluator: E, max_depth: u32, _time_limit: Duration) -> Self {
    assert!(max_depth > 0);
    MinimaxAgent {
      _s: PhantomData,
      _l: PhantomData,
      evaluator: evaluator,
      max_depth: max_depth,
    }
  }
}

impl<'a, S: State<'a>, E: Evaluator<'a, S>> Agent<'a, S>
    for MinimaxAgent<'a, S, E> {
  fn select_move(&mut self, state: &S) -> Result<Box<AgentReport<S::Move>>, &'static str> {
    if state.is_terminal() {
      return Err("Terminal state");
    }

    let report = minimax_fixed_depth(state, &self.evaluator, self.max_depth, 0.999);

    Ok(Box::new(report))
  }
}


#[cfg(test)]
mod test {

use def::{Agent, Game};
use subtractor::Subtractor;
use terminal_evaluator::TerminalEvaluator;
use super::*;

#[test]
fn subtractor() {
  let mut agent = MinimaxAgent::new(TerminalEvaluator::new(), 10, Duration::from_secs(1));
  let game = Subtractor::new(10, 4);
  let mut state = game.new_game();

  assert_eq!(2, agent.select_move(&state).unwrap().get_move());

  state.play(3).unwrap();

  assert_eq!(3, agent.select_move(&state).unwrap().get_move());
}

}