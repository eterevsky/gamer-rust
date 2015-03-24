#[derive(Debug, PartialEq)]
pub struct IPlayer(pub u8);

impl IPlayer {
  pub fn next2(self) -> Self {
    match self {
      IPlayer(0) => IPlayer(1),
      IPlayer(1) => IPlayer(0),
      _ => unreachable!()
    }
  }

  pub fn next(self, nplayers: u8) -> Self {
    let IPlayer(i) = self;
    IPlayer((i + 1) % nplayers)
  }
}

pub trait Game {
  type Move;
  type State: GameState<Self::Move>;

  fn new() -> Self::State;
}

pub trait GameState<Move> {
  fn play(&self, Move) -> Option<Self>;
  fn get_player(&self) -> IPlayer;
  fn is_terminal(&self) -> bool;
  fn get_payoff(&self, IPlayer) -> Option<i32>;
}
