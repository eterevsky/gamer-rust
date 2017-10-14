use def::{Agent, Game, State};
use gomoku::Gomoku;
use hexapawn::Hexapawn;
// use minimax::MinimaxAgent;
use registry::create_agent;
use spec::{GameSpec, AgentSpec};
use subtractor::Subtractor;

pub fn play<G: Game>(
    game: &G, player1: &mut Agent<G::State>, player2: &mut Agent<G::State>
) -> f32 {
  let mut state = game.new_game();

  while !state.is_terminal() {
    println!("{}", state);
    let report = if state.get_player() {
      player1.select_move(&state).unwrap()
    } else {
      player2.select_move(&state).unwrap()
    };
    println!("{}", report);
    state.play(report.get_move()).unwrap();
  }

  println!("{}", state);
  let payoff = state.get_payoff().unwrap();
  println!("Payoff: {}", payoff);
  payoff
}

fn play_game<G: Game>(
    game: &G, player1_spec: &AgentSpec, player2_spec: &AgentSpec
) -> f32 {
  let mut player1 = create_agent(game, player1_spec);
  let mut player2 = create_agent(game, player2_spec);
  let payoff = play(game, &mut *player1, &mut *player2);
  payoff
}

pub fn play_spec(
    game_spec: &GameSpec, player1_spec: &AgentSpec, player2_spec: &AgentSpec
) -> f32 {
  match game_spec {
    &GameSpec::Gomoku => {
      let game = Gomoku::default();
      play_game(game, player1_spec, player2_spec)
    },
    &GameSpec::Hexapawn(width, height) => {
      let game = Hexapawn::new(width, height);
      play_game(&game, player1_spec, player2_spec)
    },
    &GameSpec::Subtractor(start, max_sub) => {
      let game = Subtractor::new(start, max_sub);
      play_game(&game, player1_spec, player2_spec)
    }
  }
}

// fn train_game<'g, G: Game<'g>>(
//     game: &'g G, evaluator_spec: &EvaluatorSpec, steps: u64
// ) -> AgentSpec {
//   let mut evaluator = create_training_evaluator(game, evaluator_spec);
//   evaluator.train(steps);
//   let minimax = MinimaxAgent::new(evaluator, 1000, None);
//   minimax.spec()
// }

// pub fn train_spec(
//     game_spec: &GameSpec, evaluator_spec: &EvaluatorSpec, steps: u64
// ) -> AgentSpec {
//   match game_spec {
//     &GameSpec::Gomoku => {
//       let game = Gomoku::default();
//       train_game(game, evaluator_spec, steps)
//     },
//     &GameSpec::Hexapawn(width, height) => {
//       let game = Hexapawn::new(width, height);
//       train_game(game, evaluator_spec, steps)
//     },
//     &GameSpec::Subtractor(start, max_sub) => {
//       let game = Subtractor::new(start, max_sub);
//       train_game(game, evaluator_spec, steps)
//     }
//   }
// }
