#[cfg(test)]
use gomoku::game;

use def;
use gomoku::game::SIZE;
use gomoku::gomoku_move::GomokuMove;

const BOARD_LEN: uint = SIZE * SIZE;

pub struct GomokuState {
  stone: [bool, ..BOARD_LEN],
  color: [bool, ..BOARD_LEN],
  player: bool
}

#[cfg(test)]
#[deriving(PartialEq, Show)]
enum PointState {
  Black,
  White,
  Empty
}

impl GomokuState {
  pub fn new() -> GomokuState {
    GomokuState {
      stone: [false, ..BOARD_LEN],
      color: [false, ..BOARD_LEN],
      player: true
    }
  }

  #[cfg(test)]
  fn get(&self, p: uint) -> PointState {
    if self.stone[p] {
      if self.color[p] {
        Black
      } else {
        White
      }
    } else {
      Empty
    }
  }

  #[cfg(test)]
  fn gets(&self, s: &str) -> Option<PointState> {
    match game::idx_from_str(s) {
      Some(p) => Some(self.get(p)),
      None    => None
    }
  }
}

impl def::State<GomokuMove> for GomokuState {
  fn play(&self, gmove: GomokuMove) -> Option<GomokuState> {
    let GomokuMove(point) = gmove;

    if self.stone[point] {
      return None;
    }

    let mut new_state = GomokuState {
      stone: self.stone,
      color: self.color,
      player: !self.player
    };

    new_state.stone[point] = true;
    new_state.color[point] = self.player;

    return Some(new_state);
  }

  fn is_terminal(&self) -> bool {
    false
  }

  fn get_player(&self) -> uint {
    1
  }

  fn get_payoff(&self, player: uint) -> Option<int> {
    None
  }
}

#[test]
fn test_empty_point_in_new_gomoku() {
  let state = GomokuState::new();
  assert_eq!(Some(Empty), state.gets("A1"));
}

#[test]
fn test_gomoku_play() {
  let state0 = GomokuState::new();
    assert_eq!(Some(Empty), state0.gets("c3"));

  let state1 = state0.play(from_str("c3").unwrap()).unwrap();
  assert!(!state1.player);
  assert_eq!(Some(Black), state1.gets("c3"));
  assert!(state0.player);
  assert_eq!(Some(Empty), state0.gets("c3"));

  let state2 = state1.play(from_str("d4").unwrap()).unwrap();
  assert!(state2.player);
  assert_eq!(Some(White), state2.gets("d4"));

  assert!(state2.play(from_str("d4").unwrap()).is_none());
}
