use std::fmt;
use std::from_str;
use std::char;

use gomoku::common::GOMOKU_SIZE;

pub struct GomokuMove(pub uint);

impl GomokuMove {
  fn from_coord(col : uint, row : uint) -> GomokuMove {
    assert!(col < GOMOKU_SIZE);
    assert!(row < GOMOKU_SIZE);
    return GomokuMove(col * GOMOKU_SIZE + row);
  }
}

impl from_str::FromStr for GomokuMove {
  fn from_str(move_str : &str) -> Option<GomokuMove> {
    let mut chars = move_str.chars();
    let col : uint = match chars.next() {
      Some(c) if 'A' <= c && c < 'I' => c as uint - ('A' as uint),
      Some(c) if 'I' < c && c <= 'Z' => c as uint - ('B' as uint),
      Some(c) if 'a' <= c && c < 'i' => c as uint - ('a' as uint),
      Some(c) if 'i' < c && c <= 'z' => c as uint - ('b' as uint),
      _ => {
        return None;
      }
    };

    let mut row = 0;

    for c in chars {
      let digit = match c {
        '0' ... '9' => c as uint - ('0' as uint),
        _ => return None
      };
      row = row * 10 + digit;
    }

    return Some(GomokuMove::from_coord(col, row - 1));
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
    let col = point / GOMOKU_SIZE;
    let col_char = if col < 8 {
      char::from_u32('A' as u32 + col as u32).unwrap()
    } else {
      char::from_u32('B' as u32 + col as u32).unwrap()
    };
    let row = point % GOMOKU_SIZE + 1;
    return write!(formatter, "{}{}", col_char, row);
  }
}


#[test]
fn test_gomoku_move_parse() {
  assert_eq!(None, from_str::<GomokuMove>("abc"));
  assert_eq!(Some(GomokuMove(0)), from_str("a1"));
  assert_eq!(Some(GomokuMove(0)), from_str("A1"));
  assert_eq!(Some(GomokuMove(0)), from_str("A1"));
  assert_eq!(Some(GomokuMove(GOMOKU_SIZE * 8 + 2)), from_str("J3"));
}

#[test]
fn test_gomoku_move_to_string() {
  assert_eq!("A1", GomokuMove(0).to_string().as_slice());
  assert_eq!("A10", GomokuMove(9).to_string().as_slice());
  assert_eq!("B1", GomokuMove(GOMOKU_SIZE).to_string().as_slice());
}
