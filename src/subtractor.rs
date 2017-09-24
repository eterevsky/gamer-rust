//! A trivial game for the testing purposes. Starting with a positive integer
//! number N, each player can subtrack a number between 1 and M. Whoever gets 0
//! as a result, looses.

use rand;
use std::cmp;
use std::fmt;

use def;

pub struct Subtractor {
  start: u32,
  max_sub: u32
}

impl Subtractor {
  pub fn new(start: u32, max_sub: u32) -> Subtractor {
    Subtractor{start, max_sub}
  }
}

impl<'g> def::Game<'g> for Subtractor {
  type State = SubtractorState;
  fn new_game(&'g self) -> SubtractorState {
    SubtractorState::new(self.start, self.max_sub)
  }
}

#[derive(Clone)]
pub struct SubtractorState {
  number: u32,
  max_sub: u32,
  player: bool
}

impl SubtractorState {
  pub fn new(start: u32, max_sub: u32) -> SubtractorState {
    SubtractorState{number: start, max_sub: max_sub, player: true}
  }
}

impl<'g> def::State<'g> for SubtractorState {
  type Move = u32;

  fn get_player(&self) -> bool { self.player }
  fn is_terminal(&self) -> bool { self.number == 0 }

  fn get_payoff(&self) -> Option<f32> {
    if self.number == 0 {
      if self.player { Some(1.0) } else { Some(-1.0) }
    } else {
      None
    }
  }

  fn iter_moves<'s>(&'s self) -> Box<Iterator<Item = u32> + 's> {
    Box::new((1..cmp::min(self.max_sub, self.number + 1)))
  }

  fn get_random_move<R: rand::Rng>(&self, rng: &mut R) -> Option<Self::Move> {
    if self.number != 0 {
      Some(rng.gen_range(1, cmp::min(self.max_sub, self.number + 1)))
    } else {
      None
    }
  }

  fn play(&mut self, m: u32) -> Result<(), &'static str> {
    if 0 < m && m < self.max_sub && m <= self.number {
      self.number -= m;
      self.player = !self.player;
      Ok(())
    } else {
      Err("Subtracting wrong number")
    }
  }
}

impl fmt::Display for SubtractorState {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "Number: {}, player {}",
           self.number, if self.player { 1 } else { 2 })?;
    Ok(())
  }
}

#[cfg(test)]
mod test {

use def::Game;
use def::State;
use super::*;

#[test]
fn game() {
  let game = Subtractor::new(10, 4);
  let mut state = game.new_game();
  assert!(state.get_player());
  assert!(!state.is_terminal());
  assert!(state.get_payoff().is_none());
  assert_eq!(vec![1, 2, 3], state.iter_moves().collect::<Vec<u32>>());

  assert!(state.play(0).is_err());
  assert!(state.get_player());
  assert!(state.play(4).is_err());
  assert!(state.get_player());

  assert!(state.play(3).is_ok());
  assert!(!state.get_player());
  assert!(!state.is_terminal());

  assert!(state.play(3).is_ok());
  assert!(state.get_player());

  assert!(state.play(3).is_ok());
  assert!(!state.get_player());
  assert!(!state.is_terminal());
  assert_eq!(vec![1], state.iter_moves().collect::<Vec<u32>>());

  assert!(state.play(2).is_err());
  assert!(state.play(1).is_ok());

  assert!(state.is_terminal());
  assert_eq!(Some(1.0), state.get_payoff());
}

}