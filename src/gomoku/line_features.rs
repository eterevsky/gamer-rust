//! FeatureExtractor with features based on the continuous lines of stones
//! of the same color.

use def::State;
use feature_evaluator::FeatureExtractor;
use gomoku::gomoku::{BOARD_LEN, SIZE, GomokuState, PointState};

#[derive(Clone, Copy, Debug)]
pub struct LineRange {
  start: usize,
  step: usize,
  end: usize,
  diagonal: bool
}

/// All horizontal, vertical and diagonal lines on the board is scanned. Every
/// sequence of stones adds a fixed value to the total evaluation. A value
/// of sequence depends on:
///  - number of stones in the line,
///  - color of the stones (whether the stones belong to the player whos turn
///    it is now or now),
///  - whether there's an empty space on one or both sides of the line.
pub struct GomokuLineFeatureExtractor {
  lines: Vec<LineRange>
}

lazy_static! {
  static ref EXTRACTOR_INSTANCE: GomokuLineFeatureExtractor =
      GomokuLineFeatureExtractor::new();
}

impl GomokuLineFeatureExtractor {
  pub fn new() -> GomokuLineFeatureExtractor {
    GomokuLineFeatureExtractor {
      lines: Self::gen_lines()
    }
  }

  pub fn default() -> &'static GomokuLineFeatureExtractor {
    &*EXTRACTOR_INSTANCE
  }

  #[cfg(test)]
  pub fn gen_lines_for_test() -> Vec<LineRange> {
    Self::gen_lines()
  }

  fn gen_lines() -> Vec<LineRange> {
    let size = SIZE as usize;
    let mut lines = Vec::new();

    // Iterate through points on the top side.
    for p in 0..size {
      // Vertical line.
      lines.push(
          LineRange{start: p, step: size, end: BOARD_LEN, diagonal: false});
      // Diagonal line to bottom-right.
      lines.push(
          LineRange{
            start: p,
            step: size + 1,
            end: (size - p) * size,
            diagonal: true});
      // Diagonal line to bottom-left.
      lines.push(
          LineRange{
            start: p,
            step: size - 1,
            end: (p + 1) * size - 1,
            diagonal: true});
    }

    let mut p = 0;
    while p < BOARD_LEN {
      // Horizontal line.
      lines.push(LineRange{start: p, step: 1, end: p + size, diagonal: false});
      if p != 0 {
        // Diagonal line to bottom-right.
        lines.push(
            LineRange{
              start: p,
              step: size + 1,
              end: BOARD_LEN,
              diagonal: true});
      }
      p += size;
    }

    let mut p = 2*size - 1;
    while p < BOARD_LEN {
      // Diagonal line to bottom-left.
      lines.push(LineRange{start: p, step: size - 1, end: BOARD_LEN, diagonal: true});
      p += size;
    }

    lines
  }

  #[cfg(test)]
  pub fn encode_for_test(len: u32, both_ends: bool, diagonal: bool, active_player: bool)
      -> usize {
    Self::encode(len, both_ends, diagonal, active_player)
  }

  fn encode(len: u32, both_ends: bool, diagonal: bool, active_player: bool)
      -> usize {
    assert!(len < 5);
    assert!(len > 0);
    (len - 1 + if both_ends {4} else {0} + if diagonal {8} else {0} +
        if active_player {16} else {0}) as usize
  }
  
  fn process_single_line<'g>(state: &GomokuState<'g>, line: &LineRange,
                             features: &mut Vec<f32>) {
    let mut line_len = 0;
    let mut color = PointState::Empty;
    let mut point = line.start;
    let mut opened_line = false;
    let active_player = PointState::from_player(state.get_player());

    while point < line.end {
      if state.board[point] == color {
        line_len += 1;
      } else {
        if color != PointState::Empty {
          let open_ends = if opened_line { 1 } else { 0 } +
                          if state.board[point] == PointState::Empty { 1 }
                                                                else { 0 };
          if open_ends > 0 {
            features[Self::encode(line_len, open_ends > 1, line.diagonal,
                                  color == active_player)] += 1.0;
          }
        }
        opened_line = color == PointState::Empty && line_len > 0;
        line_len = 1;
        color = state.board[point];
      }

      point += line.step;
    }

    if color != PointState::Empty {
      let open_ends = if opened_line { 1 } else { 0 };
      if open_ends > 0 {
        features[Self::encode(line_len, open_ends > 1, line.diagonal,
                              color == active_player)] += 1.0;
      }
    }
  }
}

impl <'g> FeatureExtractor<'g, GomokuState<'g>> for GomokuLineFeatureExtractor {
  type FeatureVector = Vec<f32>;

  fn extract(&self, state: &GomokuState<'g>) -> Vec<f32> {
    // Length 1-4, 1 or 2 open ends, straight or diagonal, 
    let mut features = vec![0.0; 33];
    features[32] = 1.0;  // Bias

    for line in self.lines.iter() {
      Self::process_single_line(state, line, &mut features);
    }

    features
  }
}

#[cfg(test)]
mod test {

use gomoku::gomoku_test::run_game;
use gomoku::Gomoku;
use def::Game;
use super::*;

fn encode(len: u32, both_ends: bool, diagonal: bool, active_player: bool) -> usize {
  GomokuLineFeatureExtractor::encode_for_test(len, both_ends, diagonal, active_player)
}

#[test]
fn encode_correct() {
  let mut encodes = vec![false; 32];
  for len in 1..5 {
    for both_ends in &[false, true] {
      for diagonal in &[false, true] {
        for active_player in &[false, true] {
          let enc = encode(len, *both_ends, *diagonal, *active_player);
          assert!(!encodes[enc]);
          encodes[enc] = true;
        }
      }
    }
  }

  for v in encodes.iter() {
    assert!(v);
  }
}

#[test]
fn gen_lines() {
  let lines = GomokuLineFeatureExtractor::gen_lines_for_test();
  assert_eq!(19 + 19 + 37 + 37, lines.len());
}

#[test]
fn single_stone_empty() {
  let extractor = GomokuLineFeatureExtractor::default();
  let state = Gomoku::default().new_game();
  let features = extractor.extract(&state);
  assert_eq!(33, features.len());
  assert_eq!(1.0, features[32]);
  assert_eq!(0.0, features[0]);
}

#[test]
fn single_stone_a1() {
  let extractor = GomokuLineFeatureExtractor::default();
  let state = run_game("a1", 0.0);
  let features = extractor.extract(&state);
  assert_eq!(33, features.len());
  assert_eq!(1.0, features[encode(1, false, true, false)]);
  assert_eq!(2.0, features[encode(1, false, false, false)]);
  assert_eq!(0.0, features[encode(1, false, true, true)]);
  assert_eq!(0.0, features[encode(1, false, false, true)]);
}

#[test]
fn single_stone_a19() {
  let extractor = GomokuLineFeatureExtractor::default();
  let state = run_game("a19", 0.0);
  let features = extractor.extract(&state);
  assert_eq!(33, features.len());
  assert_eq!(1.0, features[encode(1, false, true, false)]);
  assert_eq!(2.0, features[encode(1, false, false, false)]);
}

#[test]
fn single_stone_t1() {
  let extractor = GomokuLineFeatureExtractor::default();
  let state = run_game("t1", 0.0);
  let features = extractor.extract(&state);
  assert_eq!(33, features.len());
  assert_eq!(1.0, features[encode(1, false, true, false)]);
  assert_eq!(2.0, features[encode(1, false, false, false)]);
}

#[test]
fn single_stone_t19() {
  let extractor = GomokuLineFeatureExtractor::default();
  let state = run_game("t19", 0.0);
  let features = extractor.extract(&state);
  assert_eq!(33, features.len());
  assert_eq!(1.0, features[encode(1, false, true, false)]);
  assert_eq!(2.0, features[encode(1, false, false, false)]);
}

#[test]
fn single_stone_a2() {
  let extractor = GomokuLineFeatureExtractor::default();
  let state = run_game("a2", 0.0);
  let features = extractor.extract(&state);
  assert_eq!(33, features.len());
  assert_eq!(2.0, features[encode(1, false, true, false)]);
  assert_eq!(1.0, features[encode(1, false, false, false)]);
  assert_eq!(1.0, features[encode(1, true, false, false)]);
}

#[test]
fn single_stone_c3() {
  let extractor = GomokuLineFeatureExtractor::default();
  let state = run_game("c3", 0.0);
  let features = extractor.extract(&state);
  assert_eq!(33, features.len());
  assert_eq!(0.0, features[encode(1, false, true, false)]);
  assert_eq!(0.0, features[encode(1, false, false, false)]);
  assert_eq!(2.0, features[encode(1, true, false, false)]);
  assert_eq!(2.0, features[encode(1, true, false, false)]);
}

//  . . .
//  . . X
//  . X O
//  . . .
#[test]
fn three_stones() {
  let extractor = GomokuLineFeatureExtractor::default();
  let state = run_game("t17 t16 s16", 0.0);
  let features = extractor.extract(&state);
  assert_eq!(33, features.len());
  // straight
  assert_eq!(3.0, features[encode(1, false, false, false)]);
  assert_eq!(1.0, features[encode(1, true, false, false)]);
  // diagonals
  assert_eq!(1.0, features[encode(1, false, true, false)]);
  assert_eq!(1.0, features[encode(1, true, true, false)]);
  assert_eq!(1.0, features[encode(2, false, true, false)]);
 
  // O
  assert_eq!(1.0, features[encode(1, false, false, true)]);
  assert_eq!(2.0, features[encode(1, false, true, true)]);
}

// 4 . . . . . . 
// 3 . . . . . .
// 2 O X X X X O
// 1 . . O O . .
//   a b c d e f
#[test]
fn four_blocked() {
  let extractor = GomokuLineFeatureExtractor::default();
  let state = run_game("b2 c1 c2 d1 d2 a2 e2 f2", 0.0);
  let features = extractor.extract(&state);

  // X
  assert_eq!(0.0, features[encode(4, false, false, true)]);
  assert_eq!(0.0, features[encode(4, true, false, true)]);

  // O
  assert_eq!(1.0, features[encode(2, true, false, false)]);
}

}  // mod test