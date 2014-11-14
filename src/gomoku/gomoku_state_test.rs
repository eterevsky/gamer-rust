use def::Game;
use def::GameState;
use gomoku::gomoku::Gomoku;
use gomoku::gomoku_move::GomokuMove;
use gomoku::gomoku_state::GomokuState;
use gomoku::gomoku_state::{Empty,White,Black};

#[test]
fn test_empty_point_in_new_gomoku() {
  let state = Gomoku::new().new();
  assert_eq!(Some(Empty), state.gets("A1"));
}

#[test]
fn test_gomoku_play() {
  let state0 = Gomoku::new().new();
  assert_eq!(Some(Empty), state0.gets("c3"));

  let state1 = state0.play(from_str("c3").unwrap()).unwrap();
  assert_eq!(1, state1.get_player());
  assert_eq!(Some(Black), state1.gets("c3"));
  assert_eq!(0, state0.get_player());
  assert_eq!(Some(Empty), state0.gets("c3"));

  let state2 = state1.play(from_str("d4").unwrap()).unwrap();
  assert!(state2.get_player_bool());
  assert_eq!(Some(White), state2.gets("d4"));

  assert!(state2.play(from_str("d4").unwrap()).is_none());
}

fn test_gomoku_run_game(moves_str: &str, result: int) {
  let mut state = Gomoku::new().new();
  let mut player = 0u;

  for move_str in moves_str.split_str(" ") {
    assert_eq!(player, state.get_player());
    assert!(!state.is_terminal());
    let m: GomokuMove = from_str(move_str).unwrap();
    state = state.play(m).unwrap();
    player = 1 - player;
  }

  assert!(state.is_terminal());
  assert_eq!(Some(result), state.get_payoff(0));
}

#[test]
fn test_gomoku_game1() {
  test_gomoku_run_game("a1 b1 b2 c2 c3 d3 d4 e4 e5", 1);
}
