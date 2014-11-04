use def;
use gomoku::gomoku_state::GomokuState;
use gomoku::gomoku_move::GomokuMove;

pub struct Gomoku;

impl def::Game<GomokuState, GomokuMove> for Gomoku {
  fn new() -> GomokuState {
    return GomokuState::new();
  }
}


pub const SIZE: uint = 19;

pub fn idx_from_coords(col: uint, row: uint) -> uint {
  assert!(col < SIZE);
  assert!(row < SIZE);
  col * SIZE + row
}

pub fn idx_from_str(s: &str) -> Option<uint> {
  let mut chars = s.chars();
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

  return Some(idx_from_coords(col, row - 1));
}


#[test]
fn test_create_game() {
  let state: GomokuState = def::Game::new();
  assert!(!state.is_terminal());
  assert_eq!(None, state.get_payoff());
  assert_eq!(1, state.get_player());
}
