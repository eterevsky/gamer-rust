use rand;
use regex::Regex;
use std::fmt;

use board::{Cell, Board, point_to_a};
use def::{FeatureExtractor, Game, State};
use spec::FeatureExtractorSpec;
use status::Status;

lazy_static! {
  static ref INSTANCE_3_3: Hexapawn = Hexapawn::new(3, 3);
  static ref INSTANCE_8_8: Hexapawn = Hexapawn::new(8, 8);
  static ref MOVE_RE: Regex =
      Regex::new(r"^([[:alpha:]]\d+)([-x])([[:alpha:]]\d+)$").unwrap();
}

pub struct Hexapawn {
  width: u32,
  height: u32
}

impl Hexapawn {
  pub fn new(width: u32, height: u32) -> Hexapawn {
    Hexapawn { width, height }
  }

  pub fn default(width: u32, height: u32) -> &'static Hexapawn {
    match (width, height) {
      (3, 3) => &*INSTANCE_3_3,
      (8, 8) => &*INSTANCE_8_8,
      _ => unreachable!()
    }
  }
}

impl Game for Hexapawn {
  type State = HexapawnState;

  fn new_game(&self) -> HexapawnState {
    HexapawnState::new(self.width, self.height)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct HexapawnMove {
  width: u16,
  from: u16,
  to: u16
}

impl HexapawnMove {
  fn new(from: usize, to: usize, width: u32) -> HexapawnMove {
    HexapawnMove {
      from: from as u16,
      to: to as u16,
      width: width as u16
    }
  }

  fn is_take(self) ->bool {
    self.from % self.width != self.to % self.width
  }
}

impl fmt::Display for HexapawnMove {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
        f, "{}{}{}",
        point_to_a(self.from as usize, self.width as u32),
        if self.is_take() {'x'} else {'-'},
        point_to_a(self.to as usize, self.width as u32))
  }
}

#[derive(Clone, Debug)]
pub struct HexapawnState {
  board: Board<HexapawnCell>,
  status: Status,
  moves: Vec<HexapawnMove>
}

impl HexapawnState {
  fn new(width: u32, height: u32) -> HexapawnState {
    let mut state = HexapawnState {
      board: Board::new_empty(width, height),
      status: Status::new(),
      moves: Vec::new()
    };
    for x in 0..width {
      state.board.set_xy(x, 0, HexapawnCell::White);
      state.board.set_xy(x, height - 1, HexapawnCell::Black);
    }
    state.gen_moves();
    state
  }

  fn gen_moves(&mut self) {
    self.moves.clear();
    let player = self.get_player();
    let width = self.board.width;
    for point in 0..self.board.len() {
      if self.board.get(point).unwrap().is_player(player) {
        let next_row = if player { point + width as usize }
                       else { point - width as usize };
        assert!(next_row < self.board.len(), "player {:?}, point {:?}, board {:?}", player, point, self.board);
        let (x, _) = self.board.point_to_xy(point);
        if x != 0 && self.board.get(next_row - 1).unwrap().is_player(!player) {
          self.moves.push(HexapawnMove::new(point, next_row - 1, width));
        }
        if self.board.get(next_row).unwrap().is_empty() {
          self.moves.push(HexapawnMove::new(point, next_row, width));
        }
        if x != width - 1 && self.board.get(next_row + 1).unwrap().is_player(!player) {
          self.moves.push(HexapawnMove::new(point, next_row + 1, width));
        }
      }
    }
  }

  fn check_move(&self, m: HexapawnMove) -> Result<(), &'static str> {
    let player = self.get_player();
    if !self.board.get(m.from as usize).unwrap().is_player(player) {
      return Err("Original square doesn't contain player's pawn.");
    }
    if m.is_take() {
      if !self.board.get(m.to as usize).unwrap().is_player(!player) {
        return Err("Target square doesn't contain opponent's pawn.");
      }
    } else {
      if !self.board.get(m.to as usize).unwrap().is_empty() {
        return Err("Target square is not empty.");
      }
    }

    Ok(())
  }
}

impl State for HexapawnState {
  type Move = HexapawnMove;

  fn get_player(&self) -> bool {
    self.status.get_player()
  }

  fn is_terminal(&self) -> bool {
    self.status.is_terminal()
  }

  fn get_payoff(&self) -> Option<f32> {
    self.status.get_payoff()
  }

  fn iter_moves<'s>(&'s self) -> Box<Iterator<Item=HexapawnMove> + 's> {
    Box::new(self.moves.iter().map(|&m| m))
  }

  fn get_random_move<R: rand::Rng>(&self, rng: &mut R) -> Option<HexapawnMove> {
    if self.is_terminal() {
      return None;
    }

    Some(*rng.choose(&self.moves).unwrap())
  }

  fn play(&mut self, m: HexapawnMove) -> Result<(), &'static str> {
    let player = self.get_player();
    self.check_move(m)?;
    self.board.set(m.from as usize, HexapawnCell::Empty);
    self.board.set(m.to as usize, HexapawnCell::player(player));

    self.status.switch_player();

    let (_, y) = self.board.point_to_xy(m.to as usize);
    if y == 0 || y == self.board.height - 1 {
      self.status.set_winner(player);
      return Ok(())
    }

    self.gen_moves();
    if self.moves.is_empty() {
      self.status.set_winner(player);
    }

    Ok(())
  }

  fn undo(&mut self, m: HexapawnMove) -> Result<(), &'static str> {
    let player = !self.get_player();
    if !self.board.get(m.to as usize).unwrap().is_player(player) ||
       !self.board.get(m.from as usize).unwrap().is_empty() {
      return Err("Can't undo move")
    }

    self.board.set(m.from as usize, HexapawnCell::player(player));
    self.board.set(m.to as usize, if m.is_take() { HexapawnCell::player(!player) }
                         else { HexapawnCell::Empty });
    self.status.undo_terminal();
    self.status.switch_player();

    Ok(())
  }

  fn parse_move(&self, move_str: &str) -> Result<HexapawnMove, &'static str> {
    let caps = MOVE_RE.captures(move_str).ok_or("Error parsing Hexapawn move.")?;
    let from = self.board.parse_point(&caps[1]).ok_or("Error parsing Hexapawn move.")?;
    let to = self.board.parse_point(&caps[3]).ok_or("Error parsing Hexapawn move.")?;
    let m = HexapawnMove::new(from, to, self.board.width);
    self.check_move(m)?;
    if m.is_take() == (&caps[2] == "x") {
      Ok(m)
    } else {
      Err("Incorrect `-` vs `x`")
    }
  }
}

impl fmt::Display for HexapawnState {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.board.format(false))
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HexapawnCell {
  Empty,
  White,
  Black
}

impl HexapawnCell {
  fn is_player(self, player: bool) -> bool {
    if player { self == HexapawnCell::White } else { self == HexapawnCell::Black }
  }

  fn player(player: bool) -> HexapawnCell {
    if player { HexapawnCell::White } else { HexapawnCell::Black }
  }
}

impl Cell for HexapawnCell {
  fn empty() -> HexapawnCell {
    HexapawnCell::Empty
  }

  fn is_empty(self) -> bool {
    self == HexapawnCell::Empty
  }

  fn ascii(self) -> char {
    match self {
      HexapawnCell::Empty => '.',
      HexapawnCell::White => 'W',
      HexapawnCell::Black => 'B'
    }
  }

  fn unicode(self) -> char {
    match self {
      HexapawnCell::Empty => '·',
      HexapawnCell::White => '♙',
      HexapawnCell::Black => '♟'
    }
  }
}

#[derive(Clone)]
pub struct HexapawnNumberOfPawnsExtractor {}

impl HexapawnNumberOfPawnsExtractor {
  pub fn new() -> Self { HexapawnNumberOfPawnsExtractor {} }
}

impl FeatureExtractor<HexapawnState> for HexapawnNumberOfPawnsExtractor {
  fn nfeatures(&self) -> usize { 3 }

  fn extract(&self, state: &HexapawnState) -> Vec<f32> {
    let (whites, blacks) = state.board.iter().fold(
        (0, 0),
        |(w, b), &c|
        match c {
          HexapawnCell::White => (w + 1, b),
          HexapawnCell::Black => (w, b + 1),
          _ => (w, b)
        }
    );

    if state.get_player() {
      vec![1.0, whites as f32, blacks as f32]
    } else {
      vec![1.0, blacks as f32, whites as f32]
    }
  }

  fn spec(&self) -> FeatureExtractorSpec {
    FeatureExtractorSpec::HexapawnNumberOfPawns
  }
}

#[cfg(test)]
mod test {
use super::*;

#[test]
fn init3() {
  let game = Hexapawn::new(3, 3);
  let mut state = game.new_game();

  assert_eq!(Some(HexapawnCell::White), state.board.get_a("a1"));
  assert_eq!(Some(HexapawnCell::Black), state.board.get_a("b3"));
  assert_eq!(Some(HexapawnCell::Empty), state.board.get_a("c2"));

  assert!(state.get_player());
  assert!(!state.is_terminal());

  let moves: Vec<_> = state.iter_moves().collect();
  assert_eq!(3, moves.len());

  assert!(state.play(HexapawnMove::new(1, 4, 3)).is_ok());
  assert!(!state.get_player());
  assert!(!state.is_terminal());

  let moves: Vec<_> = state.iter_moves().collect();
  assert_eq!(4, moves.len());

  assert!(state.play(HexapawnMove::new(6, 4, 3)).is_ok());
  assert!(state.get_player());
  assert!(!state.is_terminal());

  assert!(state.play(HexapawnMove::new(0, 3, 3)).is_ok());
  assert!(state.play(HexapawnMove::new(4, 1, 3)).is_ok());

  assert!(state.is_terminal());
  assert_eq!(Some(-1.0), state.get_payoff());

  assert!(state.undo(HexapawnMove::new(4, 1, 3)).is_ok());

  assert!(!state.is_terminal());
  assert!(!state.get_player());
  assert_eq!(Some(HexapawnCell::White), state.board.get_a("a2"));
  assert_eq!(Some(HexapawnCell::Black), state.board.get_a("b2"));
  assert_eq!(Some(HexapawnCell::Empty), state.board.get_a("b1"));
}

#[test]
fn stalemate() {
  let game = Hexapawn::new(3, 3);
  let mut state = game.new_game();

  assert!(state.play(HexapawnMove::new(0, 3, 3)).is_ok());
  assert!(state.play(HexapawnMove::new(7, 4, 3)).is_ok());
  assert!(state.play(HexapawnMove::new(2, 5, 3)).is_ok());

  assert!(state.is_terminal());
  assert_eq!(Some(1.0), state.get_payoff());
}

#[test]
fn random_game() {
  let game = Hexapawn::new(3, 3);
  let mut state = game.new_game();

  let mut rng = rand::XorShiftRng::new_unseeded();

  while !state.is_terminal() {
    let m = state.get_random_move(&mut rng).unwrap();
    assert!(state.play(m).is_ok());
  }

  assert!(state.is_terminal());
}

#[test]
fn extractor() {
  let mut state = Hexapawn::default(3, 3).new_game();
  let extractor = HexapawnNumberOfPawnsExtractor::new();
  assert_eq!(vec![1.0, 3.0, 3.0], extractor.extract(&state));
  let m = state.parse_move("a1-a2").unwrap();
  assert!(state.play(m).is_ok());
  assert_eq!(vec![1.0, 3.0, 3.0], extractor.extract(&state));
  let m = state.parse_move("b3xa2").unwrap();
  assert!(state.play(m).is_ok());
  assert_eq!(vec![1.0, 2.0, 3.0], extractor.extract(&state));
  let m = state.parse_move("b1-b2").unwrap();
  assert!(state.play(m).is_ok());
  assert_eq!(vec![1.0, 3.0, 2.0], extractor.extract(&state));
}

}  // mod test