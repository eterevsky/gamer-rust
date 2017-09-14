use def::Evaluator;
use def::State;
use gomoku::gomoku::GomokuState;
use gomoku::gomoku::PointState;
use gomoku::gomoku::BOARD_LEN;
use gomoku::gomoku::SIZE;

#[derive(Clone, Copy, Debug)]
pub struct GomokuEvaluator {}

impl GomokuEvaluator {
  pub fn new() -> GomokuEvaluator {
    GomokuEvaluator {}
  }
}

impl<'a> Evaluator<'a, GomokuState<'a>> for GomokuEvaluator {
  fn evaluate(&self, state: &GomokuState<'a>) -> f32 {
    if state.is_terminal() {
      state.get_payoff().unwrap()
    } else {
      0.0
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct GomokuLinesEvaluator {
  values: [f32; 16]
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
    let us = SIZE as usize;

    if state.is_terminal() {
      return state.get_payoff().unwrap() * 10000.0;
    }
    let mut value: f32 = 0.0;

    // Iterate through points on the top side.
    for p in 0..us {
      // Vertical line.
      value += self.evaluate_line(state, p, us, us, false);
      // Diagonal line to bottom-right.
      value += self.evaluate_line(state, p, us + 1, us - p, true);
      // Diagonal line to bottom-left.
      value += self.evaluate_line(state, p, us - 1, p + 1, true);
    }
    let mut p: usize = 0;
    while p < BOARD_LEN {
      // Horizontal line.
      value += self.evaluate_line(state, p, 1, us, false);
      if p != 0 {
        // Diagonal line to bottom-right.
        value += self.evaluate_line(state, p, us + 1, us - p / us, true);
      }
      p += us;
    }
    p = us - 1;
    while p < BOARD_LEN {
      // Diagonal line to bottom-left.
      value += self.evaluate_line(state, p, us - 1, us - p / us, true);
      p += us;
    }

    if state.get_player() {
      value
    } else {
      -value
    }    
  }
}

impl GomokuLinesEvaluator {
  pub fn default_instance() -> GomokuLinesEvaluator {
    GomokuLinesEvaluator {
      values: [
         0.1,   1.,  10.,    100.,
         1.,   10.,  100.,  1000.,
        -0.1,  -1.,  -10.,  -100.,
        -1.,  -10., -100., -1000.
      ]
    }

  }

  fn evaluate_line<'a>(&self, state: &GomokuState<'a>, start: usize, step: usize,
                       nsteps: usize, diagonal: bool)
      -> f32 {
    let mut value: f32 = 0.0;
    
    let mut line_len = 0;
    let mut color = PointState::Empty;
    let mut point = start;
    let mut opened_line = true;
    for _ in 0..nsteps {
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

      point += step;
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
    assert!(line_len < 5);
    assert!(line_len > 0);
    if open_ends == 0 {
      return 0.0;
    }

    let open_ends = open_ends as usize;

    let encoded: usize = if acting_player { 8 } else { 0 } +
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
fn simple_evaluator() {
  let game = Gomoku::new();
  let evaluator = GomokuEvaluator::new();
  let state = run_moves_on_state(&game, "J10");
  assert_eq!(0.0, evaluator.evaluate(&state));
}

#[test]
fn evaluator_empty() {
  let game = Gomoku::new();
  let evaluator = GomokuLinesEvaluator::default_instance();
  let state = game.new_game();
  assert_eq!(0.0, evaluator.evaluate(&state));
}

#[test]
fn evaluator_single_stone() {
  let game = Gomoku::new();
  let evaluator = GomokuLinesEvaluator::default_instance();
  let state = run_moves_on_state(&game, "J10");
  assert_eq!(-4.0, evaluator.evaluate(&state));
}

#[test]
fn evaluator_two_stones() {
  let game = Gomoku::new();
  let evaluator = GomokuLinesEvaluator::default_instance();
  let state = run_moves_on_state(&game, "J10 K10");
  assert_eq!(0.0, evaluator.evaluate(&state));
}

#[test]
fn evaluator_corner_stone() {
  let game = Gomoku::new();
  let evaluator = GomokuLinesEvaluator::default_instance();
  let state = run_moves_on_state(&game, "A1");
  assert_eq!(-0.3, evaluator.evaluate(&state));
}

}