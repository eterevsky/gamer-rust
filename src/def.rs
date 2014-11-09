pub trait Game<State: GameState<Move>, Move> {
  fn new(self) -> State;
}

pub trait GameState<Move> {
  fn play(self, Move) -> Option<Self>;
  fn get_player(&self) -> uint;
  fn is_terminal(&self) -> bool;
  fn get_payoff(&self, uint) -> Option<int>;
}
