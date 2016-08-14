//! General definitions of games, moves, players etc.

use rand::Rng;
use std::fmt;

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
  fn nplayers(&self) -> u32;
}

pub trait GameState : Game + Clone + fmt::Display {
  type Move: Copy + Clone;

  fn apply(&mut self, Self::Move) -> Result<(), &'static str>;
  fn apply_random(&mut self) -> Result<(), &'static str>;
  fn get_player(&self) -> IPlayer;
  fn get_payoff(&self, IPlayer) -> Option<f32>;
  fn is_terminal(&self) -> bool {
    self.get_payoff(0).is_some()
  }
}

pub trait MoveGenerator<S: GameState> {
  fn generate(&self, state: &S) -> Vec<S::Move>;
}

pub trait MoveSelector<S: GameState> {
  /// None if the state is terminal.
  fn select(&self, state: &S) -> Option<S::Move>;
}

pub trait Evaluator<S: GameState> {
  fn evaluate(&self, state: &S) -> f32;

  fn evaluate_move(&self, state: &S, m: S::Move) -> f32 {
    let mut state_clone = state.clone();
    state_clone.apply(m).ok();
    self.evaluate(&state_clone)
  }
}

pub trait TerminalEvaluator<S: GameState> {
  // Some(..) -- if terminal, should be compatible with payoff
  // None -- if not terminal
  fn evaluate_terminal(&self, state: &S) -> Option<f32>;
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
