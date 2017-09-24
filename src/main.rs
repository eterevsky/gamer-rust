
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
use gamer::gomoku::GomokuState;
use gamer::minimax::MiniMaxAgent;
use gamer::subtractor::Subtractor;
use gamer::terminal_evaluator::TerminalEvaluator;

fn bench_random_game<'g, G: Game<'g>>(game: &'g G) -> f64 {
  const N: u32 = 1_000_000;

  let start = time::precise_time_s();
  let mut rng = rand::XorShiftRng::new_unseeded();

  for _ in 0..N {
    let mut state: G::State = game.new_game();
    while let Some(m) = state.get_random_move(&mut rng) {
      state.play(m).unwrap();
    }
    state.get_payoff().unwrap();
  }

  let end = time::precise_time_s();
  ((end - start) as f64) / (N as f64)
}

fn bench_random_gomoku() {
  let game = Gomoku::new();
  let time = bench_random_game(&game);
  println!("bench_random_gomoku: {} us\n", 1E6 * time);
}

fn bench_random_subtractor100() {
  let game = Subtractor::new(100, 4);
  let time = bench_random_game(&game);
  println!("bench_random_subtractor100: {} us\n", 1E6 * time);
}

fn bench_minimax_subtractor() {
  let game = Subtractor::new(21, 4);
  let state = game.new_game();
  let mut agent = MiniMaxAgent::new(TerminalEvaluator::new(), 20, 1.0);
  let report = agent.select_move(&state).unwrap();

  println!("bench_minimax_subtractor:\n{}", report);
}

fn bench_minimax_gomoku_start() {
  let game = Gomoku::new();
  let state = game.new_game();
  let mut agent = MiniMaxAgent::new(GomokuLinesEvaluator::new_default(), 3, 1000.0);
  let report = agent.select_move(&state).unwrap();

  println!("bench_minimax_gomoku_start:\n{}", report);
}

fn bench_minimax_gomoku_forced() {
  let game = Gomoku::new();
  let mut state = game.new_game();
  state.play("A1".parse().unwrap()).unwrap();
  state.play("B1".parse().unwrap()).unwrap();
  state.play("A2".parse().unwrap()).unwrap();
  state.play("B2".parse().unwrap()).unwrap();
  state.play("A3".parse().unwrap()).unwrap();
  state.play("B3".parse().unwrap()).unwrap();
  state.play("A4".parse().unwrap()).unwrap();
  let mut agent = MiniMaxAgent::new(GomokuLinesEvaluator::new_default(), 3, 1000.0);
  let report = agent.select_move(&state).unwrap();

  println!("bench_minimax_gomoku_forced:\n{}", report);
}

fn bench() {
  bench_random_gomoku();
  bench_random_subtractor100();
  bench_minimax_subtractor();
  bench_minimax_gomoku_start();
  bench_minimax_gomoku_forced();
}

fn play_gomoku() {
  let game = Gomoku::new();
  let mut state: GomokuState = game.new_game();
  // let mut random_agent = RandomAgent::new(rand::XorShiftRng::new_unseeded());
  let mut player1 =
    // MiniMaxAgent::new(&GomokuLinesEvaluator::new_default(), 3, 1000.0);
    MiniMaxAgent::new(GomokuLinesEvaluator::new_default(), 3, 1000.0);
  let mut player2 =
    MiniMaxAgent::new(GomokuLinesEvaluator::new_default(), 4, 1000.0);
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

  if let Some(_) = args.subcommand_matches("bench") {
    bench();
  } else if let Some(_) = args.subcommand_matches("play") {
    play_gomoku();
  }
}
