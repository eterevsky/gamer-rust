use std::marker::PhantomData;
use std::time::{Duration, Instant};

use crate::def::{Agent, AgentReport, Evaluator, Policy, State};
use crate::spec::AgentSpec;

use super::report::MctsReport;
use super::search::MctsSearch;

pub struct MctsAgent<S: State, P: Policy<S>, E: Evaluator<S>> {
  _state: PhantomData<S>,
  policy: P,
  evaluator: E,
  max_samples: u64,
  time_limit: Option<Duration>,
}

impl<S: State, P: Policy<S>, E: Evaluator<S>> MctsAgent<S, P, E> {
  pub fn new(
    policy: P,
    evaluator: E,
    max_samples: Option<u64>,
    time_limit: Option<Duration>,
  ) -> Self {
    MctsAgent {
      _state: PhantomData,
      policy,
      evaluator,
      max_samples: max_samples.unwrap_or(1000000000),
      time_limit,
    }
  }

  pub fn select_move_with_report(
    &self,
    state: &S,
  ) -> Result<MctsReport<S::Move>, &'static str> {
    let mut search = MctsSearch::new(
      &self.policy,
      &self.evaluator,
      state.clone(),
    );
    search.search(
      self.max_samples,
      Instant::now() + self.time_limit.unwrap_or(Duration::from_secs(1000000))      
    )
  }
}

impl<S: State, P: Policy<S>, E: Evaluator<S>> Agent<S> for MctsAgent<S, P, E> {
  fn select_move(
    &self,
    state: &S,
  ) -> Result<Box<AgentReport<S::Move>>, &'static str> {
    self
      .select_move_with_report(state)
      .map(|report| Box::new(report.clone()) as Box<AgentReport<S::Move>>)
  }

  fn spec(&self) -> AgentSpec {
    panic!("not implemented")
  }
}

#[cfg(test)]
mod test {

  use crate::def::Game;
  use crate::games::Subtractor;
  use crate::equal_policy::EqualPolicy;
  use crate::evaluators::SamplerEvaluator;

  use super::*;

  #[test]
  fn play_subtractor() {
    let game = Subtractor::new(10, 4);
    let policy = EqualPolicy::new();
    let evaluator = SamplerEvaluator::new(1, 1.0);

    let agent = MctsAgent::new(policy, evaluator, Some(1000), None);

    let state = game.new_game();
    let report = agent.select_move_with_report(&state).unwrap();
    assert_eq!(2, report.get_move());
  }

} // mod test
