//! A trivial game for the testing purposes. Starting with a positive integer
//! number N, each player can subtrack a number between 1 and M. Whoever can't
//! make a move, looses.

use rand;
use std::cmp;
use std::fmt;

use def::{FeatureExtractor, Game, Regression, State};
use spec::{FeatureExtractorSpec};

lazy_static! {
  static ref INSTANCE_21_4: Subtractor = Subtractor::new(21, 4);
}

pub struct Subtractor {
  start: u32,
  max_sub: u32
}

impl Subtractor {
  pub fn new(start: u32, max_sub: u32) -> Subtractor {
    Subtractor{start, max_sub}
  }

  pub fn default(start: u32, max_sub: u32) -> &'static Subtractor {
    if (start, max_sub) == (21, 4) {
      &*INSTANCE_21_4
    } else {
      panic!()
    }
  }
}

impl Game for Subtractor {
  type State = SubtractorState;
  fn new_game(&self) -> SubtractorState {
    SubtractorState::new(self.start, self.max_sub)
  }
}

#[derive(Clone, Debug)]
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

impl State for SubtractorState {
  type Move = u32;

  fn get_player(&self) -> bool { self.player }
  fn is_terminal(&self) -> bool { self.number == 0 }

  fn get_payoff(&self) -> Option<f32> {
    if self.number == 0 {
      if self.player { Some(-1.0) } else { Some(1.0) }
    } else {
      None
    }
  }

  fn iter_moves<'s>(&'s self) -> Box<Iterator<Item = u32> + 's> {
    Box::new(1..cmp::min(self.max_sub, self.number + 1))
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

  fn undo(&mut self, m: u32) -> Result<(), &'static str> {
    self.number += m;
    self.player = !self.player;
    Ok(())
  }

  fn parse_move(&self, move_str: &str) -> Result<u32, &'static str> {
    move_str.parse().map_err(|_| "Error parsing Subtractor move.")
  }
}

impl fmt::Display for SubtractorState {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    writeln!(f, "Number: {}, player {}",
           self.number, if self.player { 1 } else { 2 })?;
    Ok(())
  }
}

/// Extracts boolean (1.0 or 0.0), based on whether the number is divisible by
/// 1, 2, 3 etc.
#[derive(Clone)]
pub struct SubtractorFeatureExtractor {
  nfeatures: u32
}

impl SubtractorFeatureExtractor {
  pub fn new(nfeatures: u32) -> SubtractorFeatureExtractor {
    SubtractorFeatureExtractor {
      nfeatures
    }
  }
}

impl FeatureExtractor<SubtractorState> for SubtractorFeatureExtractor {
  fn nfeatures(&self) -> usize { self.nfeatures as usize }

  fn extract(&self, state: &SubtractorState) -> Vec<f32> {
    (1..(self.nfeatures + 1))
        .map(|x| if state.number % (x as u32) == 0 { 1.0 } else { 0.0 })
        .collect()
  }

  fn spec(&self) -> FeatureExtractorSpec {
    FeatureExtractorSpec::Subtractor(self.nfeatures)
  }

  fn report<R: Regression>(&self, regression: &R) {
    for f in 1..(self.nfeatures + 1) {
      print!("{}: {:.3}  ", f, regression.params()[f as usize - 1]);
    }
    println!();
  }
}

#[cfg(test)]
mod test {

use def::{FeatureExtractor, Game, State};
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

  // Player 1: -3  ->  7
  assert!(state.play(3).is_ok());
  assert!(!state.get_player());
  assert!(!state.is_terminal());

  // Player 2: -3  ->  4
  assert!(state.play(3).is_ok());
  assert!(state.get_player());

  // Player 1: -3  ->  1
  assert!(state.play(3).is_ok());
  assert!(!state.get_player());
  assert!(!state.is_terminal());
  assert_eq!(vec![1], state.iter_moves().collect::<Vec<u32>>());

  assert!(state.play(2).is_err());

  // Player 2: -1  ->  0
  assert!(state.play(1).is_ok());

  assert!(state.is_terminal());
  assert_eq!(Some(-1.0), state.get_payoff());
}

#[test]
fn feature_extractor() {
  let game = Subtractor::new(10, 4);
  let state = game.new_game();
  let extractor = SubtractorFeatureExtractor::new(5);
  assert_eq!(vec![1.0, 1.0, 0.0, 0.0, 1.0], extractor.extract(&state));
}

}