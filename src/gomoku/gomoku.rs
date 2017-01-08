use std::fmt;
use std::rc::Rc;
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

pub struct Gomoku {
  lines_margins: Rc<[[LinesMargins; 4]; BOARD_LEN]>
}

impl Gomoku {
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

  fn new() -> Gomoku {
    Gomoku {
      lines_margins: Rc::new(Self::create_lines_margins())
    }
  }

  fn new_game(&self) -> GomokuState {
    GomokuState::new(self.lines_margins.clone())
  }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum PointState {
  Empty,
  Black,
  White
}

impl PointState {
  fn from_player(player: bool) -> PointState {
    if player { PointState::Black } else { PointState::White }
  }
}

pub struct GomokuState {
  board: [PointState; BOARD_LEN],
  status: u32,
  lines_margins: Rc<[[LinesMargins; 4]; BOARD_LEN]>
}

impl GomokuState {
  fn new(l: Rc<[[LinesMargins; 4]; BOARD_LEN]>) -> GomokuState {
    GomokuState {
      board: [PointState::Empty; BOARD_LEN],
      status: 1,
      lines_margins: l
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
    let lines: &[LinesMargins; 4] = &self.lines_margins[point];

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
    let player = PointState::from_player(self.get_player());

    self.board[point] = player;
    if self.player_won(point, player) {
      if self.get_player() {
        self.status |= PLAYER1_WIN_MASK;
      } else {
        self.status |= PLAYER2_WIN_MASK;
      }
    } else if self.board.iter().all(|&x| x != PointState::Empty) {
      self.status |= DRAW_MASK;
    }
    self.status ^= PLAYER_MASK;
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

    if self.board[point] == PointState::Empty {
      self.play_stone(point);
      Ok(())
    } else {
      Err("Position is taken")
    }
  }

  fn play_random_move<R: rand::Rng>(&mut self, rng: &mut R) -> Result<(), &'static str> {
    if self.is_terminal() {
      return Err("Trying to make a move in a terminal state.");
    }

    // let mut rng = rand::thread_rng();

    loop {
      let point: usize = (rng.next_u32() % BOARD_LEN as u32) as usize;
      if self.board[point] == PointState::Empty {
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

impl Clone for GomokuState {
  fn clone(&self) -> GomokuState {
    GomokuState {
      lines_margins: self.lines_margins.clone(),
      board: self.board,
      status: self.status
    }
  }
}

impl fmt::Display for GomokuState {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    try!(write!(f, "  "));
    for x in 0..SIZE {
      try!(write!(f, " {}", util::col_letter(x)));
    }
    try!(writeln!(f, ""));

    for y in 0..SIZE {
      try!(write!(f, "{:2}", SIZE - y));
      for x in 0..SIZE {
        let i = (SIZE * y + x) as usize;
        try!(match self.board[i] {
          PointState::Empty => write!(f, " ."),
          PointState::Black => write!(f, " X"),
          PointState::White => write!(f, " O")
        });

      }
      try!(writeln!(f, ""))
    }

    Ok(())
  }
}
