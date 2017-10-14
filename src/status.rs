const PLAYER_MASK: u8 = 1;
const PLAYER1_WIN_MASK: u8 = 2;
const PLAYER2_WIN_MASK: u8 = 4;
const DRAW_MASK: u8 = 8;
const TERMINAL_MASK: u8 = PLAYER1_WIN_MASK | PLAYER2_WIN_MASK | DRAW_MASK;

#[derive(Clone, Copy, Debug)]
pub struct Status(u8);

impl Status {
  pub fn new() -> Status {
    Status(0)
  }

  pub fn get_player(self) -> bool {
    self.0 & PLAYER_MASK == 0
  }

  pub fn switch_player(&mut self) {
    self.0 ^= PLAYER_MASK
  }

  pub fn is_terminal(self) -> bool {
    self.0 & TERMINAL_MASK != 0
  }

  pub fn undo_terminal(&mut self) {
    self.0 &= !TERMINAL_MASK;
  }

  pub fn set_winner(&mut self, winner: bool) {
    self.0 |= if winner {PLAYER1_WIN_MASK} else {PLAYER2_WIN_MASK};
  }

  pub fn set_draw(&mut self) {
    self.0 |= DRAW_MASK
  }

  pub fn get_payoff(self) -> Option<f32> {
    match self.0 & TERMINAL_MASK {
      PLAYER1_WIN_MASK => Some(1.0),
      PLAYER2_WIN_MASK => Some(-1.0),
      DRAW_MASK => Some(0.0),
      _ => return None
    }
  }
}


#[cfg(test)]
mod test {

use super::*;

#[test]
fn status() {
  let mut status = Status::new();
  assert!(status.get_player());
  assert!(!status.is_terminal());
  assert!(status.get_payoff().is_none());

  status.switch_player();
  assert!(!status.get_player());
  assert!(!status.is_terminal());
  assert!(status.get_payoff().is_none());

  status.set_winner(false);
  assert!(status.is_terminal());
  assert_eq!(Some(-1.0), status.get_payoff());
}

}  // mod test