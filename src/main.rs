extern crate rand;
extern crate time;
extern crate clap;

use clap::{App, Arg, SubCommand};

extern crate gamer;

use gamer::def::Game;
use gamer::def::GameState;
use gamer::gomoku::Gomoku;

fn bench<G>(game: &G) where G : Game {
  const N: u32 = 100_000;
  let mut payoff: f32 = 0.0;

  let start = time::precise_time_ns();

  for _ in 0..N {
    let mut state: G::State = game::new();
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

fn play<G>(game: &G) where G : Game {
  let mut state: G::State = game::new();
  while !state.is_terminal() {
    println!("{}", state);
    state.play_random_move();
  }
  println!("{}", state);
}

fn args_definition() -> clap::App {
  App::new("gamer").version("0.1")
      .arg(Arg::with_name("game")
               .short("g").long("game")
               .value_name("GAME")
               .takes_value(true)
               .possible_values(&["gomoku", "chess"])
               .default_value("gomoku")
               .help("Selects the game to play"))
      .subcommand(SubCommand::with_name("bench")
                             .about("Run benchmark"))
      .subcommand(SubCommand::with_name("play")
                             .about("Play a single game"))
}

fn main() {
  let args = args_definition.get_matches();

  let game: Box<Game> = match args.value_of("game") {
    "gomoku" => Box::new(Gomoku::new()),
    _ => unreachable!()
  };

  if let Some(bench_args) = args.subcommand_matches("bench") {
    bench(game);
  } else if let Some(play_args) = args.subcommand_matches("play") {
    play(game);
  }

}
