use std::fmt;
use std::char;
use std::str::FromStr;

use gomoku::gomoku;
use gomoku::gomoku::SIZE;
use gomoku::util::parse_point;

pub struct GomokuMove(pub usize);

#[derive(Debug, PartialEq)]
pub struct ParseError;

impl FromStr for GomokuMove {
  type Err = ParseError;

  fn from_str(move_str : &str) -> Result<GomokuMove, ParseError> {
    match parse_point(move_str) {
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

impl fmt::Debug for GomokuMove {
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


#[test]
fn test_gomoku_move_parse() {
  assert_eq!(Err(ParseError), FromStr::from_str("abc"));
  assert_eq!(Ok(GomokuMove(0)), FromStr::from_str("a1"));
  assert_eq!(Ok(GomokuMove(0)), FromStr::from_str("A1"));
  assert_eq!(Ok(GomokuMove(0)), FromStr::from_str("A1"));
  assert_eq!(Ok(GomokuMove(SIZE * 8 + 2)), FromStr::from_str("J3"));
}

#[test]
fn test_gomoku_move_to_string() {
  assert_eq!("A1", GomokuMove(0).to_string().as_slice());
  assert_eq!("A10", GomokuMove(9).to_string().as_slice());
  assert_eq!("B1", GomokuMove(SIZE).to_string().as_slice());
}
