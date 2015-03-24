use def::Game;
use gomoku::gomoku_state::GomokuState;
use gomoku::gomoku_move::GomokuMove;

pub struct Gomoku;

impl Gomoku {}

impl Game for Gomoku {
  type Move = GomokuMove;
  type State = GomokuState;

  fn new() -> GomokuState {
    return GomokuState::new();
  }
}

pub const SIZE: u32 = 19;
pub const BOARD_LEN: usize = (SIZE as usize) * (SIZE as usize);

#[cfg(test)]
mod test {

use def::Game;
use def::GameState;
use def::IPlayer;
use super::*;

#[test]
fn create_game() {
  let state = Gomoku::new();
  assert!(!state.is_terminal());
  assert_eq!(None, state.get_payoff(IPlayer(0)));
  assert_eq!(IPlayer(0), state.get_player());
}

}
