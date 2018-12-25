use rand::FromEntropy;
use rand::rngs::SmallRng;
use std::str::FromStr;

use crate::def::Game;
use crate::def::State;
use super::gomoku_move::GomokuMove;
use super::gomoku::{Gomoku, GomokuState, PointState, BOARD_LEN, SIZE};
use super::util;

pub fn run_game(moves_str: &str, result: f32) -> GomokuState {
  let mut state = Gomoku::default().new_game();
  let mut player = true;

  for move_str in moves_str.split(' ') {
    assert_eq!(player, state.player());
    assert!(!state.is_terminal());
    let m: GomokuMove = FromStr::from_str(move_str).unwrap();
    assert!(state.play(m).is_ok());
    player = !player;
  }

  if result != 0.0 {
    assert_eq!(Some(result), state.payoff())
  }

  state
}

#[test]
fn empty_point_in_new_game() {
  let game = Gomoku::new();
  let state = game.new_game();
  assert_eq!(Some(PointState::Empty), state.gets("A1"));
  assert_eq!(Some(PointState::Empty), state.gets("J3"));
}

#[test]
fn play() {
  let game = Gomoku::new();
  let mut state = game.new_game();
  assert_eq!(Some(PointState::Empty), state.gets("c3"));

  assert!(state.play("c3".parse().unwrap()).is_ok());
  assert_eq!(false, state.player());
  assert_eq!(Some(PointState::Black), state.gets("c3"));

  assert!(state.play("d4".parse().unwrap()).is_ok());
  assert!(state.player());
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
fn game_draw() {
  let game = Gomoku::new();
  let mut state = game.new_game();
  for x in 0..SIZE {
    for y in 0..SIZE {
      let xx = match x % 4 {
        0 => x,
        1 => x + 1,
        2 => x - 1,
        3 => x,
        _ => unreachable!()
      };

      println!("{} {}", xx, y);
      println!("{}", state);
      assert!(state.play(GomokuMove(util::xy_to_point(xx, y))).is_ok());
    }
  }

  assert!(state.is_terminal());
  assert_eq!(Some(0.0), state.payoff());
}

#[test]
fn two_moves_same_spot() {
  let game = Gomoku::new();
  let mut state = game.new_game();
  assert!(state.play("c3".parse().unwrap()).is_ok());
  assert!(state.play("c3".parse().unwrap()).is_err());
}

#[test]
fn no_moves_after_end() {
  let mut end_state = run_game("c3 d3 c4 b4 c5 c2 c6 e6 c7", 1.0);
  assert!(!end_state.play("a1".parse().unwrap()).is_ok());
}


fn is_finished(state: &GomokuState) -> bool {
  for point in 0..BOARD_LEN {
    let pstate = state.get(point);
    if pstate == PointState::Empty {
      continue;
    }
    let (xu, yu) = util::point_to_xy(point);
    let x = xu as i32;
    let y = yu as i32;

    for &(dx, dy) in [(1, 0), (1, 1), (0, 1), (-1, 1)].iter() {
      let mut tail: u32 = 1;
      for i in 1..5 {
        let xx = x + dx * i;
        let yy = y + dy * i;
        if xx < 0 || xx >= SIZE as i32 || yy < 0 || yy >= SIZE as i32 ||
           state.get(util::xy_to_point(xx as u32, yy as u32)) != pstate {
          break
        }
        tail += 1;
      }

      if tail >= 5 {
        return true
      }
    }
  }

  false
}


#[test]
fn random_game() {
  let mut rng = SmallRng::from_entropy();

  for _ in 0..100 {
    let game = Gomoku::new();
    let mut state = game.new_game();
    while !state.is_terminal() {
      assert!(!is_finished(&state));
      let m = state.get_random_move(&mut rng).unwrap();
      state.play(m).unwrap();
    }

    assert!(is_finished(&state));
    assert!(state.is_terminal());
  }
}

#[test]
fn iter_moves() {
  let state = run_game("c3 d3 c4 b4 c5 c2 c6 e6", 0.0);
  let moves: Vec<GomokuMove> = state.iter_moves().collect();
  assert_eq!(moves.len(), 19*19 - 8);
  assert!(moves.iter().find(|&&m| m == FromStr::from_str("a1").unwrap())
      .is_some());
  assert!(moves.iter().find(|&&m| m == FromStr::from_str("c3").unwrap())
      .is_none());
}

#[test]
fn undo() {
  let mut state = run_game("e6 e7 f7 f6 d8 e8 e9 d7 f8 g5 g7 h6 d10 c7 c11",
                           1.0);
  let c7 = "c7".parse().unwrap();
  let c11 = "c11".parse().unwrap();
  assert!(state.undo(c11).is_ok());
  assert!(state.undo(c7).is_ok());
  assert!(state.play(c7).is_ok());
  assert!(state.play(c11).is_ok());
  assert!(state.is_terminal());
}