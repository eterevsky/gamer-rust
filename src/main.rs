extern crate clap;
#[macro_use]
extern crate gamer;
extern crate num_cpus;

use clap::{App, Arg, ArgMatches, SubCommand};
use std::fs::File;
use std::io::Write;
use std::time::Duration;

use gamer::def::Game;
use gamer::ladder::{play_game, Ladder};
use gamer::registry::create_training;
use gamer::spec::{agent_spec_to_json, load_agent_spec, load_training_spec,
                  AgentSpec, GameSpec};

fn args_definition() -> clap::App<'static, 'static> {
  App::new("gamer")
    .version(env!("CARGO_PKG_VERSION"))
    .arg(
      Arg::with_name("game")
        .required(true)
        .short("g")
        .long("game")
        .value_name("GAME")
        .takes_value(true)
        .possible_values(&["gomoku", "hexapawn", "subtractor"])
        .help("The game to be played."),
    )
    .subcommand(
      SubCommand::with_name("play")
        .about("Play a single game")
        .arg(
          Arg::with_name("player1")
            .index(1)
            .required(true)
            .value_name("PLAYER1")
            .takes_value(true)
            .default_value("random")
            .help("Specification of the first player."),
        )
        .arg(
          Arg::with_name("player2")
            .index(2)
            .required(true)
            .value_name("PLAYER2")
            .takes_value(true)
            .default_value("random")
            .help("Specification of the second player."),
        )
        .arg(
          Arg::with_name("time_per_move")
            .short("t")
            .long("time")
            .value_name("SECONDS")
            .takes_value(true)
            .default_value("0")
            .help("Time per move in seconds. 0 for no time limit."),
        ),
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
            .help("A path to initial evaluator spec."),
        )
        .arg(
          Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("PATH")
            .takes_value(true)
            .help("A path where the evaluator will be written."),
        )
        .arg(
          Arg::with_name("steps")
            .short("s")
            .long("steps")
            .value_name("STEPS")
            .takes_value(true)
            .default_value("100000000")
            .help("Number of training steps."),
        )
        .arg(
          Arg::with_name("time_limit")
            .short("t")
            .long("time")
            .value_name("SECONDS")
            .takes_value(true)
            .default_value("0")
            .help("Time limit for training."),
        ),
    )
    .subcommand(
      SubCommand::with_name("tournament")
        .about("Tournament between agents.")
        .arg(
          Arg::with_name("AGENT")
            .index(1)
            .multiple(true)
            .required(true)
            .help("A file with agent spec."),
        )
        .arg(
          Arg::with_name("rounds")
            .short("r")
            .long("rounds")
            .value_name("NUM")
            .takes_value(true)
            .default_value("1")
            .help("Number of rounds for the tournament."),
        )
        .arg(
          Arg::with_name("time_per_move")
            .short("t")
            .long("time")
            .value_name("SECONDS")
            .takes_value(true)
            .default_value("1")
            .help("Time limit for one move."),
        )
        .arg(
          Arg::with_name("threads")
            .short("j")
            .long("threads")
            .value_name("THREADS")
            .takes_value(true)
            .default_value("0")
            .hide_default_value(true)
            .help("The number of threads to run the games in parallel. \
                   By default use all available cores.")
        )
    )
}

fn parse_time_arg(arg: Option<&str>) -> Duration {
  let t: f64 = arg.unwrap().parse().unwrap();
  Duration::new(t.trunc() as u64, (t.fract() * 1E9) as u32)
}

fn format_duration(t: Duration) -> String {
  format!("{:.1}s", t.as_secs() as f64 + t.subsec_nanos() as f64 * 1E-9)
}

fn run_play<G: Game>(game: &'static G, args: &ArgMatches) {
  let t = parse_time_arg(args.value_of("time_per_move"));
  println!("Time per move: {}", format_duration(t));
  let player1_spec =
    load_agent_spec(args.value_of("player1").unwrap(), t).unwrap();
  println!("Player 1: {:?}", player1_spec);
  let player2_spec =
    load_agent_spec(args.value_of("player2").unwrap(), t).unwrap();
  println!("Player 2: {:?}\n", player2_spec);

  play_game(game, &player1_spec, &player2_spec, true);
}

fn run_train<G: Game>(game: &'static G, args: &ArgMatches) {
  let training_spec =
    load_training_spec(args.value_of("input").unwrap()).unwrap();
  println!("Training: {:?}", training_spec);
  let steps: u64 = args.value_of("steps").unwrap().parse().unwrap();
  println!("Max steps: {}", steps);
  let t = parse_time_arg(args.value_of("time_limit"));
  println!("Time limit: {}", format_duration(t));

  let mut trainer = create_training(game, &training_spec);
  trainer.train(steps, t);
  let agent_spec = AgentSpec::Minimax {depth: 1000, time_per_move: 0.0, evaluator: trainer.build_evaluator().spec()};

  let agent_json = agent_spec_to_json(&agent_spec);
  match args.value_of("output") {
    None => println!("{}", agent_json),
    Some(path) => {
      println!("Writing agent spec to {}.", path);
      let mut f = File::create(path).unwrap();
      f.write(&agent_json.into_bytes()).unwrap();
    }
  };
}

fn run_tournament<G: Game>(game: &'static G, args: &ArgMatches) {
  let rounds: u32 = args.value_of("rounds").unwrap().parse().unwrap();
  let t = parse_time_arg(args.value_of("time_per_move"));
  println!("Time per move: {}", format_duration(t));
  let threads: usize = args.value_of("threads").unwrap().parse().unwrap();
  let threads = if threads == 0 { num_cpus::get() } else { threads };
  println!("Number of worker threads: {}", threads);
  let agents: Vec<_> = args
    .values_of("AGENT")
    .unwrap()
    .map(|a| load_agent_spec(a, t).unwrap())
    .collect();
  let mut ladder = Ladder::new(game, threads);
  for agent in agents.iter() {
    let id = ladder.add_participant(agent);
    println!("{}  {:?}", id, agent);
  }

  ladder.run_full_round(rounds);
}

fn main() {
  let args = args_definition().get_matches();
  let game_spec_str = args.value_of("game").unwrap();
  let game_spec = GameSpec::parse(game_spec_str).unwrap();
  println!("Game spec: {:?}", game_spec);

  match args.subcommand() {
    ("play", Some(subargs)) => call_with_game!(run_play, &game_spec, subargs),
    ("train", Some(subargs)) => call_with_game!(run_train, &game_spec, subargs),
    ("tournament", Some(subargs)) => {
      call_with_game!(run_tournament, &game_spec, subargs)
    }
    _ => panic!("Error parsing subcommand."),
  }
}
