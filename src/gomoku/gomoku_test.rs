extern crate rand;
use std::str::FromStr;

use def::GameState;
use gomoku::gomoku_move::GomokuMove;
use gomoku::gomoku::GomokuState;
use gomoku::gomoku::PointState;

fn run_game(moves_str: &str, result: f32) -> GomokuState {
  let mut state = GomokuState::new();
  let mut player = true;

  for move_str in moves_str.split(' ') {
    assert_eq!(player, state.get_player());
    assert!(!state.is_terminal());
    let m: GomokuMove = FromStr::from_str(move_str).unwrap();
    assert!(state.play(m).is_ok());
    player = !player;
  }

  assert!(state.is_terminal());
  assert_eq!(Some(result), state.get_payoff(true));
  return state;
}

#[test]
fn empty_point_in_new_game() {
  let state = GomokuState::new();
  assert_eq!(Some(PointState::Empty), state.gets("A1"));
  assert_eq!(Some(PointState::Empty), state.gets("J3"));
}

#[test]
fn play() {
  let mut state = GomokuState::new();
  assert_eq!(Some(PointState::Empty), state.gets("c3"));

  assert!(state.play("c3".parse().unwrap()).is_ok());
  assert_eq!(false, state.get_player());
  assert_eq!(Some(PointState::Black), state.gets("c3"));

  assert!(state.play("d4".parse().unwrap()).is_ok());
  assert!(state.get_player());
  assert_eq!(Some(PointState::White), state.gets("d4"));

  assert!(!state.play("d4".parse().unwrap()).is_ok());
}

#[test]
fn game_corner_diagonal() {
  run_game("a1 b1 b2 c2 c3 d3 d4 e4 e5", 1.0);
}

#[test]
fn game_vertical() {
  run_game("c3 c4 d3 d4 e3 e4 f3 f4 g3", 1.0);
}

#[test]
fn game_horizontal() {
  run_game("c3 d3 c4 b4 c5 c2 c6 e6 c7", 1.0);
}

#[test]
fn game_diagonal1() {
  run_game("e5 f4 f5 g5 e3 e6 f6 h6 g7 h8 h7 k8 d4 j7", -1.0);
}

#[test]
fn game_diagonal2() {
  run_game("e6 e7 f7 f6 d8 e8 e9 d7 f8 g5 g7 h6 d10 c7 c11", 1.0);
}

#[test]
fn game_borders() {
  run_game("q10 q11 r10 r11 s10 s11 t10 t11 a11 a10 b11 b10 c11 c10 d11 d10 \
            c12 c9 b13 b8 a14 a7 t15 t6 s16 s5 r17 r4 q18 q3 p19",
           1.0);
}

#[test]
fn two_moves_same_spot() {
  let mut state = GomokuState::new();
  assert!(state.play("c3".parse().unwrap()).is_ok());
  assert!(state.play("c3".parse().unwrap()).is_err());
}

#[test]
fn no_moves_after_end() {
  let mut end_state = run_game("c3 d3 c4 b4 c5 c2 c6 e6 c7", 1.0);
  assert!(!end_state.play("a1".parse().unwrap()).is_ok());
}

#[test]
fn random_game() {
  let mut state = GomokuState::new();

  while !state.is_terminal() {
    state.play_random_move();
  }

  assert!(state.is_terminal());
}