use std::fmt;
use std::char;
use std::str::FromStr;

use gomoku::gomoku::SIZE;
use gomoku::util;

#[derive(Clone, Copy, Debug)]
pub struct GomokuMove(pub usize);

#[derive(Debug, PartialEq)]
pub struct ParseError;

impl FromStr for GomokuMove {
  type Err = ParseError;

  fn from_str(move_str : &str) -> Result<GomokuMove, ParseError> {
    match util::parse_point(move_str) {
      Some(x) => Ok(GomokuMove(x)),
      None => Err(ParseError)
    }
  }
}

impl PartialEq for GomokuMove {
  fn eq(&self, other: &GomokuMove) -> bool {
    let &GomokuMove(a) = self;
    let &GomokuMove(b) = other;
    return a == b;
  }
}

impl fmt::Display for GomokuMove {
  fn fmt(&self, formatter : &mut fmt::Formatter) -> fmt::Result {
    let &GomokuMove(point) = self;
    let col = point as u32 / SIZE;
    let col_char = if col < 8 {
      char::from_u32('A' as u32 + col).unwrap()
    } else {
      char::from_u32('B' as u32 + col).unwrap()
    };
    let row = point % SIZE as usize + 1;
    return write!(formatter, "{}{}", col_char, row);
  }
}


#[cfg(test)]
mod test {

use std::string::ToString;
use gomoku::gomoku::SIZE;
use super::*;

#[test]
fn parse_errors() {
  let err: Result<GomokuMove, ParseError> = Err(ParseError);
  assert_eq!(err, "ab1".parse());
  assert_eq!(err, "a0".parse());
  assert_eq!(err, "z1".parse());
  assert_eq!(err, "b123".parse());
  assert_eq!(err, "1a".parse());
  assert_eq!(err, "aa".parse());
  assert_eq!(err, "A0".parse());
  assert_eq!(err, "a".parse());
  assert_eq!(err, "A".parse());
  assert_eq!(err, "".parse());
  assert_eq!(err, "A999999999999999999999999".parse());
}

#[test]
fn parse_legal() {
  assert_eq!(Ok(GomokuMove(0)), "a1".parse());
  assert_eq!(Ok(GomokuMove(0)), "A1".parse());
  assert_eq!(Ok(GomokuMove(9)), "A10".parse());
  assert_eq!(Ok(GomokuMove(11)), "A12".parse());
  assert_eq!(Ok(GomokuMove(SIZE as usize)), "B1".parse());
  assert_eq!(Ok(GomokuMove(SIZE as usize)), "b1".parse());
  assert_eq!(Ok(GomokuMove(SIZE as usize * 8 + 2)), "J3".parse());
}

#[test]
fn to_string() {
  assert_eq!("A1", GomokuMove(0).to_string());
  assert_eq!("A10", GomokuMove(9).to_string());
  assert_eq!("A12", GomokuMove(11).to_string());
  assert_eq!("B1", GomokuMove(SIZE as usize).to_string());
  assert_eq!("J3", GomokuMove(SIZE as usize * 8 + 2).to_string());
}

}
