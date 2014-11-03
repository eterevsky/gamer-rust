use std::fmt;
use std::from_str;
use std::char;

use def;
use gomoku::game;
use gomoku::game::SIZE;

pub struct GomokuMove(pub uint);

impl from_str::FromStr for GomokuMove {
  fn from_str(move_str : &str) -> Option<GomokuMove> {
    match game::idx_from_str(move_str) {
      Some(x) => Some(GomokuMove(x)),
      None => None
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

impl fmt::Show for GomokuMove {
  fn fmt(&self, formatter : &mut fmt::Formatter) -> fmt::Result {
    let &GomokuMove(point) = self;
    let col = point / SIZE;
    let col_char = if col < 8 {
      char::from_u32('A' as u32 + col as u32).unwrap()
    } else {
      char::from_u32('B' as u32 + col as u32).unwrap()
    };
    let row = point % SIZE + 1;
    return write!(formatter, "{}{}", col_char, row);
  }
}

impl def::Move<game::Gomoku> for GomokuMove {
}


#[test]
fn test_gomoku_move_parse() {
  assert_eq!(None, from_str::<GomokuMove>("abc"));
  assert_eq!(Some(GomokuMove(0)), from_str("a1"));
  assert_eq!(Some(GomokuMove(0)), from_str("A1"));
  assert_eq!(Some(GomokuMove(0)), from_str("A1"));
  assert_eq!(Some(GomokuMove(SIZE * 8 + 2)), from_str("J3"));
}

#[test]
fn test_gomoku_move_to_string() {
  assert_eq!("A1", GomokuMove(0).to_string().as_slice());
  assert_eq!("A10", GomokuMove(9).to_string().as_slice());
  assert_eq!("B1", GomokuMove(SIZE).to_string().as_slice());
}
