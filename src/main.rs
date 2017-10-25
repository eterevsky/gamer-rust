extern crate clap;
extern crate gamer;

use clap::{App, Arg, SubCommand};
use std::fs::File;
use std::io::Write;
use std::time::Duration;

use gamer::ladder::{ILadder, ladder_for_game};
use gamer::play::{play_spec, train_spec};
use gamer::spec::{GameSpec, agent_spec_to_json, load_agent_spec,
                  load_evaluator_spec};

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
            .default_value("100000000")
            .help("Number of training steps.")
        )
        .arg(
          Arg::with_name("time_limit")
            .short("t")
            .long("time")
            .value_name("SECONDS")
            .takes_value(true)
            .default_value("0")
            .help("Time limit for training.")
        )
    )
    .subcommand(
      SubCommand::with_name("tournament")
        .about("Tournament between agents.")
        .arg(
          Arg::with_name("AGENT")
            .index(1)
            .multiple(true)
            .required(true)
            .help("A file with agent spec.")
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
          load_agent_spec(play_args.value_of("player1").unwrap(), t).unwrap();
      let player2_spec =
          load_agent_spec(play_args.value_of("player2").unwrap(), t).unwrap();
      println!("Player 1 spec: {:?}", player1_spec);
      println!("Player 2 spec: {:?}\n", player2_spec);
      play_spec(&game_spec, &player1_spec, &player2_spec);
    },
    ("train", Some(train_args)) => {
      let evaluator_spec =
          load_evaluator_spec(train_args.value_of("input").unwrap()).unwrap();
      println!("Evaluator spec: {:?}", evaluator_spec);
      let steps: u64 = train_args.value_of("steps").unwrap().parse().unwrap();
      let time_limit: f64 = train_args.value_of("time_limit").unwrap().parse().unwrap();
      let time_limit = Duration::new(time_limit.trunc() as u64, (time_limit.fract() * 1E9) as u32);
      println!("Steps: {}", steps);
      let agent = train_spec(&game_spec, &evaluator_spec, steps, time_limit);
      let agent_json = agent_spec_to_json(&agent);
      match train_args.value_of("output") {
        None => println!("{}", agent_json),
        Some(path) => {
          let mut f = File::create(path).unwrap();
          f.write(&agent_json.into_bytes()).unwrap();
        }
      }
    },
    ("tournament", Some(tournament_args)) => {
      let agents: Vec<_> = tournament_args
          .values_of("AGENT").unwrap()
          .map(|a| load_agent_spec(a, 1.0).unwrap())
          .collect();
      let mut ladder = ladder_for_game(&game_spec);
      for (i, a) in agents.iter().enumerate() {
        println!("{}  {:?}", i, a);
        ladder.add_participant(a);
      }

      ladder.run_full_round();
    }
    _ => panic!("Unknown subcommand.")
  }
}
