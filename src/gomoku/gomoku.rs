use std::fmt;
use std::marker::PhantomData;
use rand;

use def;
use def::Game;
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

#[derive(Clone, Copy)]
struct LinesMargins {
  delta: usize,
  start: usize,
  end: usize
}

fn init_lines_margins() -> [[LinesMargins; 4]; BOARD_LEN] {
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

lazy_static! {
  static ref LINES_MARGINS: [[LinesMargins; 4]; BOARD_LEN] = init_lines_margins();
}

pub struct Gomoku {
  lines_margins: [[LinesMargins; 4]; BOARD_LEN]
}

impl<'a> Gomoku<'a> {
  fn move_till_margin(p: usize, dx: i32, dy: i32) -> usize {
    let (xu, yu) = util::point_to_xy(p);
    let mut x = xu as i32;
    let mut y = yu as i32;
    for _ in 0..4 {
      if x == 0 || x == SIZE as i32 - 1 || y == 0 || y == SIZE as i32 - 1 {
        break;
      }
      x += dx;
      y += dy;
    }

    util::xy_to_point(x as u32, y as u32)
  }
}

impl<'a> Game<'a> for Gomoku<'a> {
  type State = GomokuState<'a>;

  fn new() -> Gomoku<'a> {
    Gomoku {}
  }

  fn new_game(&'a self) -> GomokuState<'a> {
    GomokuState::new(self)
  }
}

pub struct GomokuState<'a> {
  gomoku: &'a Gomoku<'a>,
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

impl<'a> GomokuState<'a> {
  pub fn new(g: &'a Gomoku) -> GomokuState<'a> {
    GomokuState {
      gomoku: g,
      stone: [false; BOARD_LEN],
      color: [false; BOARD_LEN],
      status: 1
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

impl<'a> def::GameState<'a> for GomokuState<'a> {
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

  fn play_random_move<R: rand::Rng>(&mut self, rng: &mut R) -> Result<(), &'static str> {
    if self.is_terminal() {
      return Err("Trying to make a move in a terminal state.");
    }

    // let mut rng = rand::thread_rng();

    loop {
      let point: usize = (rng.next_u32() % BOARD_LEN as u32) as usize;
      if !self.stone[point] {
        self.play_stone(point);
        break;
      }
    }

    Ok(())
  }

  fn is_terminal(&self) -> bool {
    self.status & TERMINAL_MASK != 0
  }

  fn get_player(&self) -> bool {
    self.status & PLAYER_MASK == 1
  }

  fn get_payoff_for_player1(&self) -> Option<f32> {
    match self.status & TERMINAL_MASK {
      PLAYER1_WIN_MASK => Some(1.0),
      PLAYER2_WIN_MASK => Some(-1.0),
      DRAW_MASK => Some(0.0),
      _ => return None
    }
  }

  fn get_payoff(&self, player: bool) -> Option<f32> {
    let value = match self.get_payoff_for_player1() {
      Some(v) => v,
      None => return None
    };

    if player { Some(value) } else { Some(-value) }
  }
}

impl<'a> Clone for GomokuState<'a> {
  fn clone(&self) -> GomokuState<'a> {
    GomokuState {
      gomoku: self.gomoku,
      stone: self.stone,
      color: self.color,
      status: self.status
    }
  }
}

impl<'a> fmt::Display for GomokuState<'a> {
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
