extern crate clap;

extern crate gamer;

use clap::{App, Arg, SubCommand};
use std::time::Duration;

use gamer::def::{Agent, AgentReport, Game, State};
use gamer::gomoku::{Gomoku, GomokuLinesEvaluator, GomokuState, GomokuLineFeatureExtractor};
use gamer::hexapawn::{Hexapawn, HexapawnMove};
use gamer::feature_evaluator::{FeatureEvaluator, LinearRegression, Regression};
use gamer::minimax::MinimaxAgent;
use gamer::random_agent::RandomAgent;
use gamer::terminal_evaluator::TerminalEvaluator;

fn play_hexapawn() {
  let hexapawn = Hexapawn::new(3, 3);
  let mut state = hexapawn.new_game();

  let mut player1 = MinimaxAgent::new(TerminalEvaluator::new(), 2, Duration::from_secs(1));
  let mut player2 = RandomAgent::new();

  while !state.is_terminal() {
    let report: Box<AgentReport<HexapawnMove>> =
        if state.get_player() {
          Box::new(player1.select_move(&state).unwrap())
        } else {
          Box::new(player2.select_move(&state).unwrap())
        };

    state.play(report.get_move()).ok();
    println!("Move: {}\n{}\n{}\n", report.get_move(), report, state);
  }

  println!("Final score: {}", state.get_payoff().unwrap());
}

fn play_gomoku() {
  let game = Gomoku::new();
  let mut state: GomokuState = game.new_game();
  // let mut random_agent = RandomAgent::new(rand::XorShiftRng::new_unseeded());
  let mut player1 =
    // MinimaxAgent::new(&GomokuLinesEvaluator::new_default(), 3, 1000.0);
    MinimaxAgent::new(GomokuLinesEvaluator::new_default(), 3, Duration::from_secs(1000));
  let mut player2 =
    MinimaxAgent::new(GomokuLinesEvaluator::new_default(), 4, Duration::from_secs(1000));
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

// fn train() {
//   let game = Subtractor::new(100, 4);
//   let extractor = SubtractorFeatureExtractor::new(10);
//   let regression = LinearRegression::new(
//       iter::repeat(0.0).take(10).collect(),
//       0.01);
//   let mut evaluator = FeatureEvaluator::new(&game, extractor, regression);
//   evaluator.train(100000, 0.999, 0.1);
//   println!("{:?}", &evaluator.regression);
//   for i in 0..12 {
//     let game = Subtractor::new(i, 4);
//     let score = evaluator.evaluate(&game.new_game());
//     println!("{} {}", i, score);
//   }
// }

fn train_gomoku() {
  let extractor = GomokuLineFeatureExtractor::new();
  let regression = LinearRegression::new(vec![0.0; 33], (0.001, 0.001));
  let mut evaluator = FeatureEvaluator::new(Gomoku::default(), extractor, regression);

  let print_progress = |evaluator: &FeatureEvaluator<'static, Vec<f32>, GomokuLineFeatureExtractor, Gomoku<'static>, LinearRegression>, step| {
    if step % 100 == 0 {
      let b = &evaluator.regression.b;
      println!("Other / straight / closed: {:?}", &b[0..4]);
      println!("Other / straight / open:   {:?}", &b[4..8]);
      println!("Other / diagonal / closed: {:?}", &b[8..12]);
      println!("Other / diagonal / open:   {:?}", &b[12..16]);
      println!("Self  / straight / closed: {:?}", &b[16..20]);
      println!("Self  / straight / open:   {:?}", &b[20..24]);
      println!("Self  / diagonal / closed: {:?}", &b[24..28]);
      println!("Self  / diagonal / open:   {:?}", &b[28..32]);
      println!("Bias:                      {:?}\n", b[32]);
    }
  };

  evaluator.train(1000000, 0.999, 0.1, &print_progress);
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
    .subcommand(SubCommand::with_name("train").about(
        "Reinforcement training for the evaluator."))
}

fn main() {
  let args = args_definition().get_matches();

  if args.subcommand_matches("play").is_some() {
    play_hexapawn();
  } else if args.subcommand_matches("train").is_some() {
    train_gomoku();
  }
}
