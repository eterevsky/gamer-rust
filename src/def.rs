//! General definitions of games, moves, players etc.

use rand::Rng;

/// Represent a player in a game.
///
/// The players are counted from 0. For 2-player game the players are IPlayer(0) and IPlayer(1).
/// In games with chance, a random player is represented as IPlayer(255).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IPlayer(pub u8);

impl IPlayer {
  /// Return the first player.
  pub fn new() -> IPlayer {
    IPlayer(0)
  }

  /// Return the next play in a 2-player game without chance.
  pub fn next2(self) -> IPlayer {
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
  fn apply(&mut self, Move) -> Result<(), &'static str>;
  fn apply_random(&mut self, &mut Rng);
  fn get_player(&self) -> IPlayer;
  fn is_terminal(&self) -> bool;
  fn get_payoff(&self, IPlayer) -> Option<i32>;
}

#[cfg(test)]
mod test {

use super::*;

#[test]
fn player_next() {
  let player0: IPlayer = IPlayer::new();
  let player1: IPlayer = player0.next2();
  assert_eq!(player0, player1.next2());
}

}
