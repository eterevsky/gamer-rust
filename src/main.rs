
extern crate clap;
extern crate rand;
extern crate time;

use clap::{App, Arg, SubCommand};

extern crate gamer;

use gamer::def::Agent;
use gamer::def::AgentReport;
use gamer::def::Game;
use gamer::def::State;
use gamer::gomoku::Gomoku;
use gamer::gomoku::GomokuLinesEvaluator;
use gamer::gomoku::GomokuTerminalEvaluator;
use gamer::gomoku::GomokuState;
use gamer::minimax::MiniMaxAgent;

fn bench<'g, G: Game<'g>>(game: &'g G) {
  const N: u32 = 1_000_000;
  let mut payoff: f32 = 0.0;

  let start = time::precise_time_ns();
  let mut rng = rand::XorShiftRng::new_unseeded();

  for _ in 0..N {
    let mut state: G::State = game.new_game();
    while let Some(m) = state.get_random_move(&mut rng) {
      state.play(m).unwrap();
    }
    payoff += state.get_payoff().unwrap();
  }

  let end = time::precise_time_ns();
  let total_len = ((end - start) as f64) / 1000000000.0;

  println!("Total time: {} s", total_len);
  println!("Time per game: {} us", total_len / (N as f64) * 1000000.0);
  println!("Payoff: {}", payoff);
}

fn play_gomoku(game: &Gomoku) {
  let mut state: GomokuState = game.new_game();
  // let mut random_agent = RandomAgent::new(rand::XorShiftRng::new_unseeded());
  let mut player1 =
    // MiniMaxAgent::new(&GomokuLinesEvaluator::new_default(), 3, 1000.0);
    MiniMaxAgent::new(&GomokuTerminalEvaluator::new(), 3, 1000.0);
  let mut player2 =
    MiniMaxAgent::new(&GomokuLinesEvaluator::new_default(), 3, 1000.0);
  while !state.is_terminal() {
    let report = if state.get_player() {
      player1.select_move(&state).unwrap()
    } else {
      player2.select_move(&state).unwrap()
    };

    state.play(report.get_move()).ok();
    println!("Move: {}\n{}\n{}\n", report.get_move(), report, state);
  }

  println!("Final score: {}", state.get_payoff().unwrap());
}

fn game_main(args: clap::ArgMatches) {
  let game = Gomoku::new();

  if let Some(_) = args.subcommand_matches("bench") {
    bench(&game);
  } else if let Some(_) = args.subcommand_matches("play") {
    play_gomoku(&game);
  }
}

fn args_definition() -> clap::App<'static, 'static> {
  App::new("gamer")
    .version("0.1")
    .arg(
      Arg::with_name("game")
        .short("g")
        .long("game")
        .value_name("GAME")
        .takes_value(true)
        .possible_values(&["gomoku"])
        .default_value("gomoku")
        .help("The game to be played."),
    )
    .subcommand(SubCommand::with_name("bench").about("Run benchmark"))
    .subcommand(SubCommand::with_name("play").about("Play a single game"))
}

fn main() {
  let args = args_definition().get_matches();

  match args.value_of("game") {
    Some("gomoku") => game_main(args),
    _ => unreachable!(),
  };
}
