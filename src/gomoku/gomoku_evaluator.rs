use def::Evaluator;
use def::State;
use gomoku::gomoku::GomokuState;
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
  values: [f32; 16];
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
    let value: f32 = 0.0;

    // Iterate through points on the top side.
    for p in 0..SIZE {
      // Vertical line.
      value += self.evaluate_line(state, p, SIZE, SIZE, false);
      // Diagonal line to bottom-right.
      value += self.evaluate_line(state, p, SIZE + 1, SIZE - p, true);
      // Diagonal line to bottom-left.
      value += self.evaluate_line(state, p, SIZE - 1, p + 1, true);
    }
    let mut p = 0;
    while p < BOARD_LEN {
      // Horizontal line.
      value += self.evaluate_line(state, p, 1, false);
      if p != 0 {
        // Diagonal line to bottom-right.
        value += self.evaluate_line(state, p, SIZE + 1, p / SIZE + 1, true);
      }
      p += SIZE;
    }
    p = BOARD_LEN - SIZE + 2;
    while p < BOARD_LEN {
      // Diagonal line to top-right.
      value += self.evaluate_line(state, p, -SIZE + 1, BOARD_LEN - p, true);
      p += 1;
    }
  }
}

impl GomokuLineEvaluator {
  fn evaluate_line<'a>(&self, state: &GomokuState<'a>, start: usize, step: usize,
                       nsteps: usize, diagonal: bool)
      -> f32 {
    let mut value: f32 = 0.0;
    
    let mut line_len = 0;
    let mut color = PointState::Empty;
    let mut point = start;
    let mut opened_line = true;
    for i in 0..nsteps {
      if state.board[point] == color {
        line_len += 1;
      } else {
        if color != PointState::Empty {
          let open_ends = if opened_line { 1 } else { 0 } +
                          if state.board[point] == PointState::Empty { 1 } else { 0 };
          value += Self::evaluate_segment(
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
      value += Self::evaluate_segment(
          line_len, color == PointState::from_player(state.get_player()),
          open_ends);
    }

    value
  }

  fn evaluate_segment(&self, line_len: usize, acting_player: bool, open_ends: usize) -> f32 {
    assert!(line_len < 5);
    assert!(line_len > 0);
    if open_ends == 0 {
      return 0.0;
    }

    let encoded = if acting_player { 8 } else { 0 } +
                  (open_ends - 1) * 4 +
                  line_len - 1;
    
    self.values[encoded];
  }
}


