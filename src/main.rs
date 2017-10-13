extern crate clap;

extern crate gamer;

use clap::{App, Arg, SubCommand};

use gamer::gomoku::{Gomoku, GomokuLineFeatureExtractor};
use gamer::feature_evaluator::{FeatureEvaluator, LinearRegression, Regression};
use gamer::play::play_spec;
use gamer::spec::{AgentSpec, GameSpec, load_agent_spec};

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
        .possible_values(&["gomoku", "subtractor", "hexapawn"])
        .default_value("gomoku")
        .help("The game to be played."),
    )
    .subcommand(
      SubCommand::with_name("play")
        .about("Play a single game")
        .arg(
          Arg::with_name("player1")
            .short("1")
            .long("player1")
            .value_name("PLAYER")
            .takes_value(true)
            .default_value("random")
            .help("Specification of the first player.")
        )
        .arg(
          Arg::with_name("player2")
            .short("2")
            .long("player2")
            .value_name("PLAYER")
            .takes_value(true)
            .default_value("random")
            .help("Specification of the second player.")
        )
    )
    .subcommand(SubCommand::with_name("train").about(
        "Reinforcement training for the evaluator."))
}



fn main() {
  let args = args_definition().get_matches();
  let game_spec_str = args.value_of("game").unwrap();
  let game_spec = GameSpec::parse(game_spec_str).unwrap();

  match args.subcommand() {
    ("play", Some(play_args)) => {
      let player1_spec_str = play_args.value_of("player1").unwrap();
      let player2_spec_str = play_args.value_of("player2").unwrap();
      let player1_spec = AgentSpec::parse(player1_spec_str).unwrap();
      let player2_spec = AgentSpec::parse(player2_spec_str).unwrap();
      play_spec(&game_spec, &player1_spec, &player2_spec);
    },
    ("train", _) => {
      train_gomoku();
    },
    _ => panic!("Unknown subcommand.")
  }
}
