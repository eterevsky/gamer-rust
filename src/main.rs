extern crate rand;
extern crate time;

extern crate gamer;

use gamer::def::Game;
use gamer::def::GameState;
use gamer::def::IPlayer;
use gamer::gomoku::Gomoku;

const N: u32 = 100_000;

fn main() {
	let mut payoff = 0;
  let mut rng = rand::XorShiftRng::new_unseeded();
	
	let start = time::precise_time_ns();

	for _ in 0..N {
		let mut state = Gomoku::new();
		while !state.is_terminal() {
			state.apply_random(&mut rng);
		}
		payoff = payoff + state.get_payoff(IPlayer(0)).unwrap();
	}
	
	let end = time::precise_time_ns();
	let total_len = ((end - start) as f64) / 1000000000.0;
	
	println!("Total time: {} s", total_len);
	println!("Time per game: {} us", total_len / (N as f64) * 1000000.0);
	println!("Payoff: {}", payoff);
}