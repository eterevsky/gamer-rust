use def;
use def::GameState;
use def::IPlayer;
use gomoku::gomoku;
use gomoku::gomoku::BOARD_LEN;
use gomoku::gomoku::SIZE;
use gomoku::gomoku_move::GomokuMove;
use gomoku::util;

const PLAYER_MASK: u32 = 1;
const PLAYER1_WIN_MASK: u32 = 2;
const PLAYER2_WIN_MASK: u32 = 4;
const DRAW_MASK: u32 = 8;
const TERMINAL_MASK: u32 = PLAYER1_WIN_MASK | PLAYER2_WIN_MASK | DRAW_MASK;

pub struct GomokuState {
  stone: [bool; BOARD_LEN],
  color: [bool; BOARD_LEN],
  status: u32
}

#[derive(PartialEq, Debug)]
pub enum PointState {
  Black,
  White,
  Empty
}

impl GomokuState {
  pub fn new() -> GomokuState {
    GomokuState {
      stone: [false; BOARD_LEN],
      color: [false; BOARD_LEN],
      status: 1
    }
  }

  pub fn get(&self, p: usize) -> PointState {
    if self.stone[p] {
      if self.color[p] {
        PointState::Black
      } else {
        PointState::White
      }
    } else {
      PointState::Empty
    }
  }

  pub fn gets(&self, s: &str) -> Option<PointState> {
    match util::parse_point(s) {
      Some(p) => Some(self.get(p)),
      None    => None
    }
  }

  pub fn get_player_bool(&self) -> bool {
    self.status & PLAYER_MASK == 1
  }
//
//   fn update_status(&mut self, point: u32) {
//     let (col, row) = point_to_xy(point) as (i32, i32);
//     let player = color[point];
//     for (dx, dy) in [(1, 0), (1, 1), (0, 1), (1, -1)] {
//       let tail: u32 = 1;
//       for i in range(1, 5) {
//         let c = col + dx * i;
//       }
//
//     }
//   }
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
    // new_state.update_status(point);

    return Some(new_state);
  }

  fn is_terminal(&self) -> bool {
    self.status & TERMINAL_MASK != 0
  }

  fn get_player(&self) -> IPlayer {
    if self.get_player_bool() {IPlayer(0)} else {IPlayer(1)}
  }

  fn get_payoff(&self, player: IPlayer) -> Option<i32> {
    let value = match self.status & TERMINAL_MASK {
      PLAYER1_WIN_MASK => 1,
      PLAYER2_WIN_MASK => -1,
      DRAW_MASK => 0,
      _ => return None
    };

    match player {
      IPlayer(0) => Some(value),
      IPlayer(1) => Some(-value),
      _ => unreachable!()
    }
  }
}
