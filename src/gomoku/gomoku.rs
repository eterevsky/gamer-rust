use std::fmt;
use rand;
use rand::distributions::{IndependentSample, Range};

use def;
use def::GameState;
use gomoku::gomoku_move::GomokuMove;
use gomoku::util;

pub const SIZE: u32 = 19;
pub const BOARD_LEN: usize = (SIZE as usize) * (SIZE as usize);
const PLAYER_MASK: u32 = 1;
const PLAYER1_WIN_MASK: u32 = 2;
const PLAYER2_WIN_MASK: u32 = 4;
const DRAW_MASK: u32 = 8;
const TERMINAL_MASK: u32 = PLAYER1_WIN_MASK | PLAYER2_WIN_MASK | DRAW_MASK;

lazy_static! {
  static ref RANGE_DIST: Range<usize> = Range::new(0, BOARD_LEN);
}

pub struct GomokuState {
  stone: [bool; BOARD_LEN],
  color: [bool; BOARD_LEN],
  status: u32
}

#[derive(PartialEq, Debug)]
#[cfg(test)]
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


  pub fn play_random_move(&mut self) {
    if self.is_terminal() {
      panic!("Trying to make a move in a terminal state.");
    }

    let mut rng = rand::thread_rng();

    loop {
      let point: usize = RANGE_DIST.ind_sample(&mut rng);
      if !self.stone[point] {
        self.play_stone(point);
        break;
      }
    }
  }

  #[cfg(test)]
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

  #[cfg(test)]
  pub fn gets(&self, s: &str) -> Option<PointState> {
    match util::parse_point(s) {
      Some(p) => Some(self.get(p)),
      None    => None
    }
  }

  fn update_status(&mut self, point: usize) {
    let (x, y) = util::point_to_xy(point);
    let (col, row) = (x as i32, y as i32);
    let player = self.color[point];
    assert!(self.stone[point]);

    for &(dx, dy) in [(1, 0), (1, 1), (0, 1), (1, -1)].iter() {
      let mut tail: u32 = 1;
      for i in 1..5 {
        let c = col + dx * i;
        let r = row + dy * i;
        if c < 0 || c >= SIZE as i32 || r < 0 || r >= SIZE as i32 {
          break
        }
        let p = util::xy_to_point(c as u32, r as u32);
        if !self.stone[p] || self.color[p] != player {
          break
        }
        tail += 1;
      }

      for i in 1..5 {
        let c = col - dx * i;
        let r = row - dy * i;
        if c < 0 || c >= SIZE as i32 || r < 0 || r >= SIZE as i32 {
          break
        }
        let p = util::xy_to_point(c as u32, r as u32);
        if !self.stone[p] || self.color[p] != player {
          break
        }
        tail += 1;
      }

      if tail >= 5 {
        if player {
          self.status |= PLAYER1_WIN_MASK;
        } else {
          self.status |= PLAYER2_WIN_MASK;
        }
      }
    }
  }

  fn play_stone(&mut self, point: usize) {
    self.stone[point] = true;
    self.color[point] = self.get_player();
    self.status ^= PLAYER_MASK;
    self.update_status(point);
  }
}

impl def::GameState for GomokuState {
  type Move = GomokuMove;
  type Player = bool;

  fn play(&mut self, gmove: GomokuMove) -> Result<(), &'static str> {
    if self.status & TERMINAL_MASK != 0 {
      return Err("Trying to make a move in a terminal state.")
    }

    let GomokuMove(point) = gmove;

    if self.stone[point] {
      Err("Position is taken")
    } else {
      self.play_stone(point);
      Ok(())
    }
  }

  fn is_terminal(&self) -> bool {
    self.status & TERMINAL_MASK != 0
  }

  fn get_player(&self) -> bool {
    self.status & PLAYER_MASK == 1
  }

  fn get_payoff(&self, player: bool) -> Option<f32> {
    let value = match self.status & TERMINAL_MASK {
      PLAYER1_WIN_MASK => 1.0,
      PLAYER2_WIN_MASK => -1.0,
      DRAW_MASK => 0.0,
      _ => return None
    };

    if player { Some(value) } else { Some(-value) }
  }
}

impl def::Game for GomokuState {
  fn nplayers(&self) -> u32 {
    2
  }
}

impl Clone for GomokuState {
  fn clone(&self) -> GomokuState {
    GomokuState {
      stone: self.stone,
      color: self.color,
      status: self.status
    }
  }
}

impl fmt::Display for GomokuState {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    for y in 0..SIZE {
      for x in 0..SIZE {
        let i = (SIZE * y + x) as usize;
        if self.stone[i] {
          if self.color[i] {
            try!(write!(f, " X"));
          } else {
            try!(write!(f, " O"));
          }
        } else {
          try!(write!(f, " ."));
        }

      }
      try!(writeln!(f, ""))
    }

    Ok(())
  }
}
