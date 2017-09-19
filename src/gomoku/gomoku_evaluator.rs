use def::Evaluator;
use def::State;
use gomoku::gomoku::GomokuState;
use gomoku::gomoku::PointState;
use gomoku::gomoku::BOARD_LEN;
use gomoku::gomoku::SIZE;

#[derive(Clone, Copy, Debug)]
pub struct GomokuTerminalEvaluator {}

impl GomokuTerminalEvaluator {
  pub fn new() -> GomokuTerminalEvaluator {
    GomokuTerminalEvaluator {}
  }
}

impl<'a> Evaluator<'a, GomokuState<'a>> for GomokuTerminalEvaluator {
  fn evaluate(&self, state: &GomokuState<'a>) -> f32 {
    if state.is_terminal() {
      state.get_payoff().unwrap()
    } else {
      0.0
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct LineRange {
  start: usize,
  step: usize,
  end: usize,
  diagonal: bool
}

#[derive(Clone, Debug)]
pub struct GomokuLinesEvaluator {
  values: [f32; 16],
  lines: Vec<LineRange>
}

/// Evaluate the position, base on lines of 1 to 5 stones of the same color.
///
/// All horizontal, vertical and diagonal lines on the board is scanned. Every
/// sequence of stones adds a fixed value to the total evaluation. A value
/// of sequence depends on:
///  - number of stones in the line,
///  - color of the stones (whether the stones belong to the player whos turn
///    it is now or now),
///  - whether there's an empty space on one or both sides of the line.
impl<'a> Evaluator<'a, GomokuState<'a>> for GomokuLinesEvaluator {
  fn evaluate(&self, state: &GomokuState<'a>) -> f32 {
    if state.is_terminal() {
      return state.get_payoff().unwrap() * 10000.0;
    }
    let mut value: f32 = 0.0;

    for line in self.lines.iter() {
      value += self.evaluate_line(state, line);
    }

    if state.get_player() {
      value
    } else {
      -value
    }
  }
}

impl GomokuLinesEvaluator {
  pub fn new_default() -> GomokuLinesEvaluator {
    GomokuLinesEvaluator {
      values: [
         0.1,   1.,  10.,    100.,
         1.,   10.,  100.,  1000.,
        -0.1,  -1.,  -10.,  -100.,
        -1.,  -10., -100., -1000.
      ],
      lines: Self::gen_lines()
    }
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

  fn evaluate_line<'a>(&self, state: &GomokuState<'a>, line: &LineRange) -> f32 {
    let mut value: f32 = 0.0;

    let mut line_len = 0;
    let mut color = PointState::Empty;
    let mut point = line.start;
    let mut opened_line = false;

    while point < line.end {
      if state.board[point] == color {
        line_len += 1;
      } else {
        if color != PointState::Empty {
          let open_ends = if opened_line { 1 } else { 0 } +
                          if state.board[point] == PointState::Empty { 1 }
                                                                else { 0 };
          value += self.evaluate_segment(
              line_len, color == PointState::from_player(state.get_player()),
              open_ends);
        }
        opened_line = color == PointState::Empty && line_len > 0;
        line_len = 1;
        color = state.board[point];
      }

      point += line.step;
    }

    if color != PointState::Empty {
      let open_ends = if opened_line { 1 } else { 0 };
      value += self.evaluate_segment(
          line_len, color == PointState::from_player(state.get_player()),
          open_ends);
    }

    value
  }

  fn evaluate_segment(&self, line_len: usize, acting_player: bool,
                      open_ends: u32) -> f32 {
    println!("evaluate_segment {} {} {}", line_len, acting_player, open_ends);
    assert!(line_len < 5);
    assert!(line_len > 0);
    if open_ends == 0 {
      return 0.0;
    }

    let open_ends = open_ends as usize;

    let encoded: usize = if acting_player { 0 } else { 8 } +
                         (open_ends - 1) * 4 +
                         line_len - 1;

    self.values[encoded]
  }
}


#[cfg(test)]
mod test {

use def::Game;
use gomoku::gomoku::Gomoku;
use gomoku::gomoku_test::run_moves_on_state;
use super::*;

#[test]
fn terminal_evaluator() {
  let game = Gomoku::new();
  let evaluator = GomokuTerminalEvaluator::new();
  let state = run_moves_on_state(&game, "J10");
  assert_eq!(0.0, evaluator.evaluate(&state));
}

#[test]
fn gen_lines() {
  let lines = GomokuLinesEvaluator::gen_lines_for_test();
  assert_eq!(19 + 19 + 37 + 37, lines.len());
}

#[test]
fn evaluator_empty() {
  let game = Gomoku::new();
  let evaluator = GomokuLinesEvaluator::new_default();
  let state = game.new_game();
  assert_eq!(0.0, evaluator.evaluate(&state));
}

#[test]
fn evaluator_single_stone() {
  let game = Gomoku::new();
  let evaluator = GomokuLinesEvaluator::new_default();
  let state = run_moves_on_state(&game, "J10");
  assert_eq!(4.0, evaluator.evaluate(&state));
}

#[test]
fn evaluator_two_stones() {
  let game = Gomoku::new();
  let evaluator = GomokuLinesEvaluator::new_default();
  let state = run_moves_on_state(&game, "J10 K10");
  assert_eq!(0.0, evaluator.evaluate(&state));
}

#[test]
fn evaluator_corner_stone() {
  let game = Gomoku::new();
  let evaluator = GomokuLinesEvaluator::new_default();
  let state = run_moves_on_state(&game, "A1");
  assert_eq!(0.3, evaluator.evaluate(&state));
}

#[test]
fn evaluator_two_corner_stones() {
  let game = Gomoku::new();
  let evaluator = GomokuLinesEvaluator::new_default();
  let state = run_moves_on_state(&game, "A1 B1");
  assert_eq!(0.2 - 0.4, evaluator.evaluate(&state));
}

#[test]
fn evaluator_three_corner_stones() {
  let game = Gomoku::new();
  let evaluator = GomokuLinesEvaluator::new_default();
  let state = run_moves_on_state(&game, "A1 A19 B1");
  assert!((1.2 -  evaluator.evaluate(&state)).abs() < 0.1);
}

}