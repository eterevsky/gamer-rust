//! FeatureExtractor with features based on the continuous lines of stones
//! of the same color.

use def::{FeatureExtractor, Regression, State};
use super::gomoku::{GomokuState, PointState, BOARD_LEN, SIZE};
use spec::{FeatureExtractorSpec};

#[derive(Clone, Copy, Debug)]
pub struct LineRange {
  start: usize,
  step: usize,
  end: usize,
  diagonal: bool,
}

/// All horizontal, vertical and diagonal lines on the board is scanned. Every
/// sequence of stones adds a fixed value to the total evaluation. A value
/// of sequence depends on:
///  - number of stones in the line,
///  - color of the stones (whether the stones belong to the player whos turn
///    it is now or now),
///  - whether there's an empty space on one or both sides of the line.
#[derive(Clone)]
pub struct GomokuLineFeatureExtractor {
  lines: Vec<LineRange>,
  min_len: usize,
}

lazy_static! {
  static ref EXTRACTOR_INSTANCES: Vec<GomokuLineFeatureExtractor> =
      vec![
          GomokuLineFeatureExtractor::new(1),
          GomokuLineFeatureExtractor::new(2),
          GomokuLineFeatureExtractor::new(3)];

}

impl GomokuLineFeatureExtractor {
  pub fn new(min_len: usize) -> GomokuLineFeatureExtractor {
    GomokuLineFeatureExtractor {
      lines: Self::gen_lines(),
      min_len,
    }
  }

  pub fn default(min_len: usize) -> &'static GomokuLineFeatureExtractor {
    let instances = &*EXTRACTOR_INSTANCES;
    &instances[min_len - 1]
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
      lines.push(LineRange {
        start: p,
        step: size,
        end: BOARD_LEN,
        diagonal: false,
      });
      // Diagonal line to bottom-right.
      lines.push(LineRange {
        start: p,
        step: size + 1,
        end: (size - p) * size,
        diagonal: true,
      });
      // Diagonal line to bottom-left.
      lines.push(LineRange {
        start: p,
        step: size - 1,
        end: (p + 1) * size - 1,
        diagonal: true,
      });
    }

    let mut p = 0;
    while p < BOARD_LEN {
      // Horizontal line.
      lines.push(LineRange {
        start: p,
        step: 1,
        end: p + size,
        diagonal: false,
      });
      if p != 0 {
        // Diagonal line to bottom-right.
        lines.push(LineRange {
          start: p,
          step: size + 1,
          end: BOARD_LEN,
          diagonal: true,
        });
      }
      p += size;
    }

    let mut p = 2 * size - 1;
    while p < BOARD_LEN {
      // Diagonal line to bottom-left.
      lines.push(LineRange {
        start: p,
        step: size - 1,
        end: BOARD_LEN,
        diagonal: true,
      });
      p += size;
    }

    lines
  }

  fn encode(
    &self,
    len: usize,
    both_ends: bool,
    diagonal: bool,
    active_player: bool,
  ) -> Option<usize> {
    assert!(len < 5);
    if len < self.min_len {
      None
    } else {
      Some(
        (len - self.min_len) * 8 + if both_ends { 1 } else { 0 } + if diagonal {
          2
        } else {
          0
        } + if active_player { 4 } else { 0 },
      )
    }
  }

  fn process_single_line(
    &self,
    state: &GomokuState,
    line: &LineRange,
    features: &mut Vec<f32>,
  ) {
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
          let open_ends = if opened_line { 1 } else { 0 }
            + if state.board[point] == PointState::Empty {
              1
            } else {
              0
            };
          if open_ends > 0 {
            if let Some(encoded) = self.encode(
              line_len,
              open_ends > 1,
              line.diagonal,
              color == active_player,
            ) {
              features[encoded] += 1.0;
            }
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
        if let Some(encoded) = self.encode(
          line_len,
          open_ends > 1,
          line.diagonal,
          color == active_player,
        ) {
          features[encoded] += 1.0;
        }
      }
    }
  }
}

impl FeatureExtractor<GomokuState> for GomokuLineFeatureExtractor {
  fn nfeatures(&self) -> usize {
    (5 - self.min_len) * 8 + 1
  }

  fn extract(&self, state: &GomokuState) -> Vec<f32> {
    // Length 1-4, 1 or 2 open ends, straight or diagonal,
    let mut features = vec![0.0; self.nfeatures()];
    features[self.nfeatures() - 1] = 1.0; // Bias

    for line in self.lines.iter() {
      self.process_single_line(state, line, &mut features);
    }

    features
  }

  fn spec(&self) -> FeatureExtractorSpec {
    FeatureExtractorSpec::GomokuLines(self.min_len as u32)
  }

  fn report<R: Regression>(&self, regression: &R) {
    let b = regression.params();
    println!(
      "closed straight / closed diagonal / open straight / open diagonal"
    );
    for &player in &[true, false] {
      for len in self.min_len..5 {
        println!(
          "{} {}: {:>6.3} {:>6.3} {:>6.3} {:>6.3}",
          (if player { "self " } else { "other" }),
          len,
          b[self.encode(len, false, false, player).unwrap()],
          b[self.encode(len, false, true, player).unwrap()],
          b[self.encode(len, true, false, player).unwrap()],
          b[self.encode(len, true, true, player).unwrap()]
        );
      }
    }
    println!("bias: {:.3}\n", b[self.nfeatures() - 1]);
  }
}

#[cfg(test)]
mod test {

  use super::super::gomoku_test::run_game;
  use super::super::gomoku::Gomoku;
  use def::Game;
  use super::*;

  fn encode(
    len: u32,
    both_ends: bool,
    diagonal: bool,
    active_player: bool,
  ) -> usize {
    GomokuLineFeatureExtractor::default(1)
      .encode(len as usize, both_ends, diagonal, active_player)
      .unwrap()
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
    let extractor = GomokuLineFeatureExtractor::default(1);
    let state = Gomoku::default().new_game();
    let features = extractor.extract(&state);
    assert_eq!(33, features.len());
    assert_eq!(1.0, features[32]);
    assert_eq!(0.0, features[0]);
  }

  #[test]
  fn single_stone_a1() {
    let extractor = GomokuLineFeatureExtractor::default(1);
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
    let extractor = GomokuLineFeatureExtractor::default(1);
    let state = run_game("a19", 0.0);
    let features = extractor.extract(&state);
    assert_eq!(33, features.len());
    assert_eq!(1.0, features[encode(1, false, true, false)]);
    assert_eq!(2.0, features[encode(1, false, false, false)]);
  }

  #[test]
  fn single_stone_t1() {
    let extractor = GomokuLineFeatureExtractor::default(1);
    let state = run_game("t1", 0.0);
    let features = extractor.extract(&state);
    assert_eq!(33, features.len());
    assert_eq!(1.0, features[encode(1, false, true, false)]);
    assert_eq!(2.0, features[encode(1, false, false, false)]);
  }

  #[test]
  fn single_stone_t19() {
    let extractor = GomokuLineFeatureExtractor::default(1);
    let state = run_game("t19", 0.0);
    let features = extractor.extract(&state);
    assert_eq!(33, features.len());
    assert_eq!(1.0, features[encode(1, false, true, false)]);
    assert_eq!(2.0, features[encode(1, false, false, false)]);
  }

  #[test]
  fn single_stone_a2() {
    let extractor = GomokuLineFeatureExtractor::default(1);
    let state = run_game("a2", 0.0);
    let features = extractor.extract(&state);
    assert_eq!(33, features.len());
    assert_eq!(2.0, features[encode(1, false, true, false)]);
    assert_eq!(1.0, features[encode(1, false, false, false)]);
    assert_eq!(1.0, features[encode(1, true, false, false)]);
  }

  #[test]
  fn single_stone_c3() {
    let extractor = GomokuLineFeatureExtractor::default(1);
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
    let extractor = GomokuLineFeatureExtractor::default(1);
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
    let extractor = GomokuLineFeatureExtractor::default(1);
    let state = run_game("b2 c1 c2 d1 d2 a2 e2 f2", 0.0);
    let features = extractor.extract(&state);

    // X
    assert_eq!(0.0, features[encode(4, false, false, true)]);
    assert_eq!(0.0, features[encode(4, true, false, true)]);

    // O
    assert_eq!(1.0, features[encode(2, true, false, false)]);
  }

  #[test]
  fn nfeatures() {
    assert_eq!(33, GomokuLineFeatureExtractor::default(1).nfeatures());
    assert_eq!(25, GomokuLineFeatureExtractor::default(2).nfeatures());
    assert_eq!(17, GomokuLineFeatureExtractor::default(3).nfeatures());
  }

  #[test]
  fn encode_oob() {
    assert_eq!(
      None,
      GomokuLineFeatureExtractor::default(2).encode(1, true, false, false)
    );
  }

  // 4 . . . . . .
  // 3 . . . . . .
  // 2 O X X X X O
  // 1 . . O O . .
  //   a b c d e f
  #[test]
  fn four_blocked_min2() {
    let extractor = GomokuLineFeatureExtractor::default(2);
    let state = run_game("b2 c1 c2 d1 d2 a2 e2 f2", 0.0);
    let features = extractor.extract(&state);

    assert_eq!(25, features.len());

    // X
    assert_eq!(
      0.0,
      features[extractor.encode(4, false, false, true).unwrap()]
    );
    assert_eq!(
      0.0,
      features[extractor.encode(4, true, false, true).unwrap()]
    );

    // O
    assert_eq!(
      1.0,
      features[extractor.encode(2, true, false, false).unwrap()]
    );
  }

} // mod test
