pub trait Game<S: State<M>, M: Move> {
  fn new() -> S;
}

pub trait State<M: Move> {
  fn play(&self, M) -> Option<Self>;
  fn get_player(&self) -> uint;
  fn is_terminal(&self) -> bool;
  fn get_payoff(&self, uint) -> Option<int>;
}

pub trait Move {
}
