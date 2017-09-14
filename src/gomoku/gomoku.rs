use std::fmt;
use std::marker::PhantomData;
use rand;

use def;
use def::Game;
use def::State;
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

pub struct Gomoku<'a> {
  _marker: PhantomData<&'a Gomoku<'a>>,
  lines_margins: [[LinesMargins; 4]; BOARD_LEN]
}

impl<'a> Gomoku<'a> {
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

impl<'a> Game<'a> for Gomoku<'a> {
  type State = GomokuState<'a>;

  fn new() -> Gomoku<'a> {
    Gomoku {
      _marker: PhantomData,
      lines_margins: Self::create_lines_margins()
    }
  }

  fn new_game(&'a self) -> GomokuState<'a> {
    GomokuState::new(self)
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

pub struct GomokuState<'a> {
  gomoku: &'a Gomoku<'a>,
  pub board: [PointState; BOARD_LEN],
  status: u32
}

impl<'a> GomokuState<'a> {
  fn new(gomoku: &'a Gomoku) -> GomokuState<'a> {
    GomokuState {
      gomoku: gomoku,
      board: [PointState::Empty; BOARD_LEN],
      status: 1
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

impl<'a> def::State<'a> for GomokuState<'a> {
  type Move = GomokuMove;

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

  fn iter_moves<'b>(&'b self) -> Box<Iterator<Item=GomokuMove> + 'b> {
    Box::new((0..BOARD_LEN).filter(move |&i| {self.board[i] == PointState::Empty})
                           .map(|i| GomokuMove(i)))
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

  fn get_payoff(&self) -> Option<f32> {
    match self.status & TERMINAL_MASK {
      PLAYER1_WIN_MASK => Some(1.0),
      PLAYER2_WIN_MASK => Some(-1.0),
      DRAW_MASK => Some(0.0),
      _ => return None
    }
  }
}

impl<'a> Clone for GomokuState<'a> {
  fn clone(&self) -> GomokuState<'a> {
    GomokuState {
      gomoku: self.gomoku,
      board: self.board,
      status: self.status
    }
  }
}

impl<'a> fmt::Display for GomokuState<'a> {
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
