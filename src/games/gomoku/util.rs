use std::char;

use super::gomoku::SIZE;

pub fn xy_to_point(col: u32, row: u32) -> usize {
  (row * SIZE + col) as usize
}

pub fn point_to_xy(point: usize) -> (u32, u32) {
  (point as u32 % SIZE, point as u32 / SIZE)
}

pub fn col_letter(col: u32) -> char {
  if col < 8 {
    char::from_u32('A' as u32 + col).unwrap()
  } else {
    char::from_u32('B' as u32 + col).unwrap()
  }
}

pub fn point_to_a(point: usize) -> String {
  let (x, y) = point_to_xy(point);
  format!("{}{}", col_letter(x), y + 1)
}

pub fn parse_point(s: &str) -> Option<usize> {
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
    if row > SIZE {
      return None
    }
  }

  if 1 <= row && col < SIZE {
    return Some(xy_to_point(col, row - 1));
  } else {
    return None
  }
}


#[cfg(test)]
mod test {

use super::super::gomoku::BOARD_LEN;
use super::*;

#[test]
fn point_to_xy_to_point() {
  for point in 0..BOARD_LEN {
    let (x, y) = point_to_xy(point);
    assert_eq!(point, xy_to_point(x, y));
  }
}

#[test]
fn point_to_a_to_point() {
  for point in 0..BOARD_LEN {
    let s = point_to_a(point);
    assert_eq!(Some(point), parse_point(&s));
  }
}

#[test]
fn parse_point_samples() {
  assert_eq!(Some(0), parse_point("A1"));
  assert_eq!(Some(18), parse_point("T1"));
  assert_eq!(Some(361 - 19), parse_point("A19"));
  assert_eq!(Some(360), parse_point("T19"));
}

}
