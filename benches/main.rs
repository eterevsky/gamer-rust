#[macro_use]
extern crate bencher;
extern crate rand;

extern crate gamer;

use bencher::Bencher;

use gamer::def::{Agent, Evaluator, Game, State};
use gamer::feature_evaluator::{FeatureEvaluator, FeatureExtractor, LinearRegression, Regression};
use gamer::gomoku::{Gomoku, GomokuState, GomokuLineFeatureExtractor};
use gamer::minimax::MiniMaxAgent;
use gamer::subtractor::{Subtractor, SubtractorFeatureExtractor};
use gamer::terminal_evaluator::TerminalEvaluator;

fn generate_random_gomoku_position() -> GomokuState<'static> {
  let mut rng = rand::XorShiftRng::new_unseeded();
  let mut state = Gomoku::default().new_game();
  for _ in 0..60 {
    let m = state.get_random_move(&mut rng).unwrap();
    state.play(m).unwrap();
  }
  state
}

fn gomoku_random(bench: &mut Bencher) {
  let game = Gomoku::new();
  let mut rng = rand::weak_rng();
  bench.iter(|| {
    let mut state = game.new_game();
    while let Some(m) = state.get_random_move(&mut rng) {
      state.play(m).unwrap();
    }
    state.get_payoff().unwrap()
  })
}

fn gomoku_lines_feature_extractor_start(bench: &mut Bencher) {
  let state = Gomoku::default().new_game();
  let feature_extractor = GomokuLineFeatureExtractor::default();
  bench.iter(|| {feature_extractor.extract(&state)});
}

fn gomoku_lines_feature_extractor_rand_position(bench: &mut Bencher) {
  let feature_extractor = GomokuLineFeatureExtractor::default();
  let state = generate_random_gomoku_position();
  bench.iter(|| {feature_extractor.extract(&state)});
}

fn gomoku_train_evaluator_1000(bench: &mut Bencher) {
  let game = Gomoku::default();

  bench.iter(|| {
    let extractor = GomokuLineFeatureExtractor::default();
    let regression = LinearRegression::new(vec![0.0; 33], (0.001, 0.0001));
    let mut evaluator = FeatureEvaluator::new(game, extractor, regression);
    evaluator.train(1000, 0.999, 0.1);
    evaluator
  });
}

fn subtractor_random(bench: &mut Bencher) {
  let game = Subtractor::new(21, 4);
  let mut rng = rand::weak_rng();
  bench.iter(|| {
    let mut state = game.new_game();
    while let Some(m) = state.get_random_move(&mut rng) {
      state.play(m).unwrap();
    }
    state.get_payoff().unwrap()
  })
}

fn subtractor_minimax(bench: &mut Bencher) {
  let state = Subtractor::new(21, 4).new_game();
  let mut agent = MiniMaxAgent::new(TerminalEvaluator::new(), 10, 1000.0);

  bench.iter(|| {agent.select_move(&state).unwrap()})
}

fn subtractor_feature_evaluator(bench: &mut Bencher) {
  let game = Subtractor::new(21, 4);
  let extractor = SubtractorFeatureExtractor::new(10);
  let regression = LinearRegression::new(
      vec![0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
      (0.0, 0.0));
  let evaluator = FeatureEvaluator::new(&game, extractor, regression);

  let state = game.new_game();
  bench.iter(|| {evaluator.evaluate(&state)});
}

fn subtractor_train_evaluator_1000(bench: &mut Bencher) {
  let game = Subtractor::new(21, 4);

  bench.iter(|| {
    let extractor = SubtractorFeatureExtractor::new(10);
    let regression = LinearRegression::new(vec![0.0; 10], (0.1, 0.001));
    let mut evaluator = FeatureEvaluator::new(&game, extractor, regression);
    evaluator.train(1000, 0.999, 0.1);
    evaluator
  });
}

benchmark_group!(benches,
    gomoku_random,
    gomoku_lines_feature_extractor_start,
    gomoku_lines_feature_extractor_rand_position,
    gomoku_train_evaluator_1000,
    subtractor_random,
    subtractor_minimax,
    subtractor_feature_evaluator,
    subtractor_train_evaluator_1000);
benchmark_main!(benches);