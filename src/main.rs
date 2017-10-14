extern crate clap;

extern crate gamer;

use clap::{App, Arg, SubCommand};

use gamer::play::play_spec;
use gamer::spec::{GameSpec, load_agent_spec, load_evaluator_spec};

// fn train_gomoku() {
//   let extractor = GomokuLineFeatureExtractor::new();
//   let regression = LinearRegression::new(vec![0.0; 33], (0.001, 0.001));
//   let mut evaluator = FeatureEvaluator::new(Gomoku::default(), extractor, regression);

//   let print_progress = |evaluator: &FeatureEvaluator<'static, Vec<f32>, GomokuLineFeatureExtractor, Gomoku<'static>, LinearRegression>, step| {
//     if step % 100 == 0 {
//       let b = &evaluator.regression.b;
//       println!("Other / straight / closed: {:?}", &b[0..4]);
//       println!("Other / straight / open:   {:?}", &b[4..8]);
//       println!("Other / diagonal / closed: {:?}", &b[8..12]);
//       println!("Other / diagonal / open:   {:?}", &b[12..16]);
//       println!("Self  / straight / closed: {:?}", &b[16..20]);
//       println!("Self  / straight / open:   {:?}", &b[20..24]);
//       println!("Self  / diagonal / closed: {:?}", &b[24..28]);
//       println!("Self  / diagonal / open:   {:?}", &b[28..32]);
//       println!("Bias:                      {:?}\n", b[32]);
//     }
//   };

//   evaluator.train(1000000, 0.999, 0.1, &print_progress);
// }

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
        .arg(
          Arg::with_name("time_per_move")
            .short("t")
            .long("time")
            .value_name("SECONDS")
            .takes_value(true)
            .default_value("0")
            .help("Time per move in seconds. 0 for no time limit.")
        )
    )
    .subcommand(
      SubCommand::with_name("train")
        .about("Reinforcement training of the evaluator.")
        .arg(
          Arg::with_name("input")
            .short("i")
            .long("input")
            .value_name("PATH")
            .takes_value(true)
            .required(true)
            .help("A path to initial evaluator spec.")
        )
        .arg(
          Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("PATH")
            .takes_value(true)
            .help("A path where the evaluator will be written.")
        )
        .arg(
          Arg::with_name("steps")
            .short("s")
            .long("steps")
            .value_name("STEPS")
            .takes_value(true)
            .default_value("10000")
            .help("Number of training steps.")
        )
    )
}

fn main() {
  let args = args_definition().get_matches();
  let game_spec_str = args.value_of("game").unwrap();
  let game_spec = GameSpec::parse(game_spec_str).unwrap();
  println!("Game spec: {:?}", game_spec);

  match args.subcommand() {
    ("play", Some(play_args)) => {
      let t: f64 = play_args.value_of("time_per_move").unwrap().parse().unwrap();
      let player1_spec =
          load_agent_spec(play_args.value_of("player1").unwrap()).unwrap();
      let player2_spec =
          load_agent_spec(play_args.value_of("player2").unwrap()).unwrap();
      println!("Player 1 spec: {:?}", player1_spec);
      println!("Player 2 spec: {:?}\n", player2_spec);
      play_spec(&game_spec, &player1_spec, &player2_spec, t);
    },
    ("train", Some(train_args)) => {
      let evaluator_spec =
          load_evaluator_spec(train_args.value_of("input").unwrap()).unwrap();
      println!("Evaluator spec: {:?}\n", evaluator_spec);
      // let steps: u64 = train_args.value_of("steps").unwrap().parse().unwrap();
      // let agent = train_spec(&game_spec, &evaluator_spec, );
    },
    _ => panic!("Unknown subcommand.")
  }
}
