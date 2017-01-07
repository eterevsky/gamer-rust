extern crate rand;
extern crate time;

extern crate gamer;

use gamer::def::GameState;
use gamer::gomoku::GomokuState;

const N: u32 = 100_000;

fn main() {
  let mut payoff: f32 = 0.0;

  let start = time::precise_time_ns();

  for _ in 0..N {
    let mut state = GomokuState::new();
    while !state.is_terminal() {
      state.play_random_move();
    }
    payoff = payoff + state.get_payoff(true).unwrap();
  }

  let end = time::precise_time_ns();
  let total_len = ((end - start) as f64) / 1000000000.0;

  println!("Total time: {} s", total_len);
  println!("Time per game: {} us", total_len / (N as f64) * 1000000.0);
  println!("Payoff: {}", payoff);
}
