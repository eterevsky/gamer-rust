use std::char;
use std::fmt;
use std::iter;

pub trait Cell: Copy + fmt::Debug {
  fn empty() -> Self;
  fn is_empty(self) -> bool;
  fn ascii(self) -> char;
  fn unicode(self) -> char;
}

#[derive(Clone, Debug)]
pub struct Board<C: Cell> {
  pub width: u32,
  pub height: u32,
  data: Vec<C>
}

impl<C: Cell> Board<C> {
  pub fn new_empty(width: u32, height: u32) -> Self {
    assert!(width <= 25);
    assert!(height <= 25);
    Board {
      width,
      height,
      data: iter::repeat(C::empty()).take((width * height) as usize).collect()
    }
  }

  pub fn xy_to_point(&self, col: u32, row: u32) -> usize {
    (row * self.width + col) as usize
  }

  pub fn point_to_xy(&self, point: usize) -> (u32, u32) {
    point_to_xy(point, self.width)
  }

  pub fn parse_point(&self, s: &str) -> Option<usize> {
    let mut chars = s.chars();
    let col: u32 = match chars.next() {
      Some(c) if 'A' <= c && c < 'I' => c as u32 - ('A' as u32),
      Some(c) if 'I' < c && c <= 'Z' => c as u32 - ('B' as u32),
      Some(c) if 'a' <= c && c < 'i' => c as u32 - ('a' as u32),
      Some(c) if 'i' < c && c <= 'z' => c as u32 - ('b' as u32),
      _ => {
        return None;
      }
    };

    let mut row = 0;

    for c in chars {
      let digit = match c {
        '0' ... '9' => c as u32 - ('0' as u32),
        _ => return None
      };
      row = row * 10 + digit;
      if row > self.height {
        return None
      }
    }

    if 1 <= row && col < self.width {
      return Some(self.xy_to_point(col, row - 1));
    } else {
      return None
    }
  }

  pub fn get(&self, point: usize) -> Option<C> {
    match self.data.get(point) {
      Some(&x) => Some(x),
      None => None
    }
  }

  pub fn get_a(&self, a: &str) -> Option<C> {
    if let Some(point) = self.parse_point(a) {
      self.get(point)
    } else {
      None
    }
  }

  pub fn get_xy(&self, x: u32, y: u32) -> Option<C> {
    self.get(self.xy_to_point(x, y))
  }

  pub fn set(&mut self, point: usize, c: C) -> Option<()> {
    if point < self.data.len() {
      self.data[point] = c;
      Some(())
    } else {
      None
    }
  }

  pub fn set_xy(&mut self, x: u32, y: u32, c: C) -> Option<()> {
    let point = self.xy_to_point(x, y);
    self.set(point, c)
  }


  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn format<'a>(&'a self, ascii: bool) -> BoardFormatter<'a, C> {
    BoardFormatter {
      board: self,
      ascii
    }
  }
}

pub struct BoardFormatter<'a, C: Cell + 'a> {
  // _c: PhantomData<C>,
  board: &'a Board<C>,
  ascii: bool
}

impl<'a, C: Cell> fmt::Display for BoardFormatter<'a, C> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "   ")?;
    for x in 0..self.board.width {
      write!(f, " {}", col_letter(x))?;
    }
    writeln!(f)?;
    for y in (0..self.board.height).rev() {
      write!(f, "{:>2}", y + 1)?;
      for x in 0..self.board.width {
        let cell = self.board.get_xy(x, y).unwrap();
        write!(f, " {}", if self.ascii { cell.ascii() } else { cell.unicode() })?;
      }
      writeln!(f, " {}", y + 1)?;
    }
    for x in 0..self.board.width {
      write!(f, " {}", col_letter(x))?;
    }
    writeln!(f)
  }
}

pub fn col_letter(col: u32) -> char {
  if col < 8 {
    char::from_u32('a' as u32 + col).unwrap()
  } else {
    char::from_u32('b' as u32 + col).unwrap()
  }
}

pub fn point_to_a(point: usize, width: u32) -> String {
  let (x, y) = point_to_xy(point, width);
  format!("{}{}", col_letter(x), y + 1)
}

pub fn point_to_xy(point: usize, width: u32) -> (u32, u32) {
  (point as u32 % width, point as u32 / width)
}


#[cfg(test)]
mod test {

use super::*;

#[test]
fn point_to_a_() {
  assert_eq!("a1", point_to_a(0, 1));
  assert_eq!("a1", point_to_a(0, 8));
  assert_eq!("a1", point_to_a(0, 19));

  assert_eq!("a11", point_to_a(10, 1));
  assert_eq!("c2", point_to_a(10, 8));
  assert_eq!("l1", point_to_a(10, 19));

  assert_eq!("t19", point_to_a(19*19 - 1, 19));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GoCell {
  Empty,
  Black,
  White
}

impl Cell for GoCell {
  fn empty() -> GoCell {
    GoCell::Empty
  }

  fn is_empty(self) -> bool {
    self == GoCell::Empty
  }

  fn ascii(self) -> char {
    match self {
      GoCell::Empty => '.',
      GoCell::Black => 'X',
      GoCell::White => 'O',
    }
  }

  fn unicode(self) -> char {
    match self {
      GoCell::Empty => '·',
      GoCell::Black => '⚫',
      GoCell::White => '⚪',
    }
  }
}

#[test]
fn board() {
  let mut board = Board::new_empty(19, 19);
  let letters: Vec<char> = "abcdefghjklmnopqrst".chars().collect();

  assert_eq!(None, board.get(19 * 19));
  for row in 0..19 {
    for col in 0..19 {
      let point = board.xy_to_point(col, row);
      assert_eq!(
          Some(point),
          board.parse_point(
              &format!("{}{}", letters[col as usize], row + 1)
          )
      );
      assert_eq!(Some(GoCell::Empty), board.get(point));
    }
  }

  board.set_xy(3, 3, GoCell::Black);
  assert_eq!(Some(GoCell::Black), board.get_xy(3, 3));

  board.set_xy(3, 15, GoCell::White);
  assert_eq!(Some(GoCell::White), board.get_xy(3, 15));

  assert!(!format!("{}", board.format(false)).is_empty());
}

}  // mod test