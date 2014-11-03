trait Game {
  fn new() -> uint;
}

trait Move<G: Game> {
}

trait State<G: Game> {
  fn play(&self, Move<G>) -> Option<Self>;
  fn get_player(&self) -> uint;
  fn is_terminal(&self) -> bool;
  fn get_payoff(&self, uint) -> Option<int>;
}
