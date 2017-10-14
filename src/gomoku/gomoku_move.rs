use std::fmt;
use std::str::FromStr;

use gomoku::util;

#[derive(Clone, Copy, Debug)]
pub struct GomokuMove(pub usize);

impl FromStr for GomokuMove {
  type Err = &'static str;

  fn from_str(move_str : &str) -> Result<GomokuMove, &'static str> {
    match util::parse_point(move_str) {
      Some(x) => Ok(GomokuMove(x)),
      None => Err("Error parsing gomoku move.")
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
    return write!(formatter, "{}", util::point_to_a(point));
  }
}


#[cfg(test)]
mod test {

use std::string::ToString;
use gomoku::gomoku::SIZE;
use super::*;

#[test]
fn parse_errors() {
  assert!("ab1".parse::<GomokuMove>().is_err());
  assert!("a0".parse::<GomokuMove>().is_err());
  assert!("z1".parse::<GomokuMove>().is_err());
  assert!("b123".parse::<GomokuMove>().is_err());
  assert!("1a".parse::<GomokuMove>().is_err());
  assert!("aa".parse::<GomokuMove>().is_err());
  assert!("A0".parse::<GomokuMove>().is_err());
  assert!("a".parse::<GomokuMove>().is_err());
  assert!("A".parse::<GomokuMove>().is_err());
  assert!("".parse::<GomokuMove>().is_err());
  assert!("A999999999999999999999999".parse::<GomokuMove>().is_err());
}

#[test]
fn parse_legal() {
  assert_eq!(Ok(GomokuMove(342)), "a19".parse());
  assert_eq!(Ok(GomokuMove(342)), "A19".parse());
  assert_eq!(Ok(GomokuMove(9)), "K1".parse());
  assert_eq!(Ok(GomokuMove(11)), "M1".parse());
  assert_eq!(Ok(GomokuMove(SIZE as usize)), "A2".parse());
  assert_eq!(Ok(GomokuMove(SIZE as usize)), "a2".parse());
  assert_eq!(Ok(GomokuMove(SIZE as usize * 10 + 2)), "C11".parse());
}

#[test]
fn to_string() {
  assert_eq!("A1", GomokuMove(0).to_string());
  assert_eq!("K1", GomokuMove(9).to_string());
  assert_eq!("M1", GomokuMove(11).to_string());
  assert_eq!("A2", GomokuMove(SIZE as usize).to_string());
  assert_eq!("C11", GomokuMove(SIZE as usize * 10 + 2).to_string());
}

}
