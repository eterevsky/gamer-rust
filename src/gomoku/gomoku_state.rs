use gomoku::common::GOMOKU_POINTS;
use gomoku::gomoku_move::GomokuMove;

pub struct GomokuState {
  stone: [bool, ..GOMOKU_POINTS],
  color: [bool, ..GOMOKU_POINTS],
  player: bool
}

impl GomokuState {
  pub fn new() -> GomokuState {
    GomokuState {
      stone: [false, ..GOMOKU_POINTS],
      color: [false, ..GOMOKU_POINTS],
      player: true
    }
  }

  pub fn play(&self, gmove: GomokuMove) -> Option<GomokuState> {
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

}

#[test]
fn test_empty_point_in_new_gomoku() {
  let state = GomokuState::new();
  assert!(!state.stone[0]);
}

#[test]
fn test_gomoku_play() {
  let state0 = GomokuState::new();
  assert!(!state0.stone[0]);

  let state1 = state0.play(GomokuMove(15)).unwrap();
  assert!(!state1.player);
  assert!(state1.stone[15]);
  assert!(state1.color[15]);
  assert!(state0.player);
  assert!(!state0.stone[15]);

  let state2 = state1.play(GomokuMove(10)).unwrap();
  assert!(state2.player);
  assert!(state2.stone[10]);
  assert!(!state2.color[10]);

  assert!(state2.play(GomokuMove(15)).is_none());
}
