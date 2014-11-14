use def;
use def::GameState;
use gomoku::gomoku;
use gomoku::gomoku::SIZE;
use gomoku::gomoku_move::GomokuMove;

const BOARD_LEN: uint = SIZE * SIZE;

const PLAYER_MASK: uint = 1;
const PLAYER1_WIN_MASK: uint = 2;
const PLAYER2_WIN_MASK: uint = 4;
const DRAW_MASK: uint = 8;
const TERMINAL_MASK: uint = PLAYER1_WIN_MASK | PLAYER2_WIN_MASK | DRAW_MASK;

pub struct GomokuState {
  stone: [bool, .. BOARD_LEN],
  color: [bool, .. BOARD_LEN],
  status: uint
}

#[cfg(test)]
#[deriving(PartialEq, Show)]
pub enum PointState {
  Black,
  White,
  Empty
}

impl GomokuState {
  pub fn new() -> GomokuState {
    GomokuState {
      stone: [false, .. BOARD_LEN],
      color: [false, .. BOARD_LEN],
      status: 1
    }
  }

  #[cfg(test)]
  pub fn get(&self, p: uint) -> PointState {
    if self.stone[p] {
      if self.color[p] {
        Black
      } else {
        White
      }
    } else {
      Empty
    }
  }

  #[cfg(test)]
  pub fn gets(&self, s: &str) -> Option<PointState> {
    match gomoku::idx_from_str(s) {
      Some(p) => Some(self.get(p)),
      None    => None
    }
  }

  pub fn get_player_bool(&self) -> bool {
    self.status & PLAYER_MASK == 1
  }
}

impl def::GameState<GomokuMove> for GomokuState {
  fn play(self, gmove: GomokuMove) -> Option<GomokuState> {
    let GomokuMove(point) = gmove;

    if self.stone[point] {
      return None;
    }

    let mut new_state = GomokuState {
      stone: self.stone,
      color: self.color,
      status: self.status ^ PLAYER_MASK
    };

    new_state.stone[point] = true;
    new_state.color[point] = self.get_player_bool();

    return Some(new_state);
  }

  fn is_terminal(&self) -> bool {
    (self.status & TERMINAL_MASK) != 0
  }

  fn get_player(&self) -> uint {
    if self.get_player_bool() {0} else {1}
  }

  fn get_payoff(&self, player: uint) -> Option<int> {
    let value = match self.status & TERMINAL_MASK {
      PLAYER1_WIN_MASK => 1,
      PLAYER2_WIN_MASK => -1,
      DRAW_MASK => 0,
      _ => return None
    };

    if player == 0 {Some(value)} else {Some(-value)}
  }
}
