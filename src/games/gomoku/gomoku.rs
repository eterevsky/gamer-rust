use lazy_static::lazy_static;
use rand;
use std::fmt;
use std::str::FromStr;

use crate::def::{Game, State};
use super::gomoku_move::GomokuMove;
use super::util;
use crate::status::Status;

pub const SIZE: u32 = 19;
pub const BOARD_LEN: usize = (SIZE as usize) * (SIZE as usize);

lazy_static! {
  static ref GOMOKU_INSTANCE: Gomoku = Gomoku::new();
}

#[derive(Clone, Copy)]
struct LinesMargins {
  delta: usize,
  start: usize,
  end: usize
}

pub struct Gomoku {
  lines_margins: [[LinesMargins; 4]; BOARD_LEN]
}

impl Gomoku {
  pub fn new() -> Gomoku {
    Gomoku {
      lines_margins: Self::create_lines_margins()
    }
  }

  pub fn default() -> &'static Gomoku {
    &*GOMOKU_INSTANCE
  }

  fn move_till_margin(p: usize, dx: i32, dy: i32) -> usize {
    let (xu, yu) = util::point_to_xy(p);
    let mut x = xu as i32;
    let mut y = yu as i32;
    for _ in 0..4 {
      if dx < 0 && x == 0 ||
         dx > 0 && x == SIZE as i32 - 1 ||
         dy < 0 && y == 0 ||
         dy > 0 && y == SIZE as i32 - 1 {
        break;
      }
      x += dx;
      y += dy;
    }

    util::xy_to_point(x as u32, y as u32)
  }

  fn create_lines_margins() -> [[LinesMargins; 4]; BOARD_LEN] {
    let mut margins: [[LinesMargins; 4]; BOARD_LEN] =
        [[LinesMargins{delta: 0, start: 0, end: 0}; 4]; BOARD_LEN];

    for p in 0..BOARD_LEN {
      margins[p][0] = LinesMargins{
          delta: 1,
          start: Gomoku::move_till_margin(p, -1, 0),
          end: Gomoku::move_till_margin(p, 1, 0)
      };
      margins[p][1] = LinesMargins{
          delta: SIZE as usize,
          start: Gomoku::move_till_margin(p, 0, -1),
          end: Gomoku::move_till_margin(p, 0, 1)
      };
      margins[p][2] = LinesMargins{
          delta: SIZE as usize - 1,
          start: Gomoku::move_till_margin(p, 1, -1),
          end: Gomoku::move_till_margin(p, -1, 1)
      };
      margins[p][3] = LinesMargins{
          delta: SIZE as usize + 1,
          start: Gomoku::move_till_margin(p, -1, -1),
          end: Gomoku::move_till_margin(p, 1, 1)
      };
    }

    margins
  }
}

impl Game for Gomoku {
  type State = GomokuState;

  fn new_game(&self) -> GomokuState {
    GomokuState::new()
  }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum PointState {
  Empty,
  Black,
  White
}

impl PointState {
  pub fn from_player(player: bool) -> PointState {
    if player { PointState::Black } else { PointState::White }
  }
}

pub struct GomokuState {
  gomoku: &'static Gomoku,
  pub board: [PointState; BOARD_LEN],
  status: Status
}

impl GomokuState {
  fn new() -> GomokuState {
    GomokuState {
      gomoku: Gomoku::default(),
      board: [PointState::Empty; BOARD_LEN],
      status: Status::new()
    }
  }

  #[cfg(test)]
  pub fn get(&self, p: usize) -> PointState {
    self.board[p]
  }

  #[cfg(test)]
  pub fn gets(&self, s: &str) -> Option<PointState> {
    match util::parse_point(s) {
      Some(p) => Some(self.get(p)),
      None    => None
    }
  }

  fn player_won(&mut self, point: usize, player: PointState) -> bool {
    debug_assert!(self.board[point] == player);
    let lines: &[LinesMargins; 4] = &self.gomoku.lines_margins[point];

    for line in lines {
      let mut len = 1;
      let mut p = point;

      while p > line.start {
        p -= line.delta;
        if self.board[p] != player {
          break;
        }
        len += 1;
      }

      p = point;
      while p < line.end {
        p += line.delta;
        if self.board[p] != player {
          break;
        }
        len += 1;
      }

      if len >= 5 {
        return true;
      }
    }

    false
  }

  fn play_stone(&mut self, point: usize) {
    let player = self.player();
    let player_stone = PointState::from_player(player);

    self.board[point] = player_stone;
    if self.player_won(point, player_stone) {
      self.status.set_winner(player);
    } else if self.board.iter().all(|&x| x != PointState::Empty) {
      self.status.set_draw();
    }
    self.status.switch_player()
  }
}

struct GomokuMoveIterator<'s> {
  state: &'s GomokuState,
  point: usize
}

impl<'s> Iterator for GomokuMoveIterator<'s> {
  type Item = GomokuMove;

  fn next(&mut self) -> Option<GomokuMove> {
    while self.point < BOARD_LEN &&
          self.state.board[self.point] != PointState::Empty {
      self.point += 1;
    }
    if self.point < BOARD_LEN {
      let current_point = self.point;
      self.point += 1;
      Some(GomokuMove(current_point))
    } else {
      None
    }
  }
}

impl State for GomokuState {
  type Move = GomokuMove;

  fn play(&mut self, gmove: GomokuMove) -> Result<(), &'static str> {
    if self.is_terminal() {
      return Err("Trying to make a move in a terminal state.")
    }

    let GomokuMove(point) = gmove;

    if self.board[point] == PointState::Empty {
      self.play_stone(point);
      Ok(())
    } else {
      Err("Position is taken")
    }
  }

  fn undo(&mut self, gmove: GomokuMove) -> Result<(), &'static str> {
    let GomokuMove(point) = gmove;
    if self.board[point] == PointState::Empty {
      Err("This wasn't the last move")
    } else {
      self.board[point] = PointState::Empty;
      self.status.switch_player();
      self.status.undo_terminal();
      Ok(())
    }
  }

  fn iter_moves<'s>(&'s self) -> Box<Iterator<Item = GomokuMove> + 's> {
    Box::new(GomokuMoveIterator{state: self, point: 0})
  }

  fn get_random_move<R: rand::Rng>(&self, rng: &mut R) -> Option<GomokuMove> {
    if self.is_terminal() {
      return None
    }

    loop {
      let point: usize = (rng.next_u32() % BOARD_LEN as u32) as usize;
      if self.board[point] == PointState::Empty {
        return Some(GomokuMove(point))
      }
    }
  }

  fn is_terminal(&self) -> bool {
    self.status.is_terminal()
  }

  fn player(&self) -> bool {
    self.status.player()
  }

  fn payoff(&self) -> Option<f32> {
    self.status.payoff()
  }

  fn parse_move(&self, move_str: &str) -> Result<GomokuMove, &'static str> {
    GomokuMove::from_str(move_str)
  }
}

impl Clone for GomokuState {
  fn clone(&self) -> GomokuState {
    GomokuState {
      gomoku: self.gomoku,
      board: self.board,
      status: self.status
    }
  }
}

impl fmt::Display for GomokuState {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "  ")?;
    for x in 0..SIZE {
      write!(f, " {}", util::col_letter(x))?;
    }
    writeln!(f, "")?;

    for y in (0..SIZE).rev() {
      write!(f, "{:2}", y + 1)?;
      for x in 0..SIZE {
        let i = (SIZE * y + x) as usize;
        match self.board[i] {
          PointState::Empty => write!(f, " ."),
          PointState::Black => write!(f, " X"),
          PointState::White => write!(f, " O")
        }?;
      }
      writeln!(f, "")?
    }

    write!(f, "  ")?;
    for x in 0..SIZE {
      write!(f, " {}", util::col_letter(x))?;
    }
    writeln!(f)?;

    Ok(())
  }
}
