#[macro_use]
extern crate bencher;
extern crate rand;

extern crate gamer;

use bencher::Bencher;
use rand::Rng;

use gamer::def::{Evaluator, Game, State};
use gamer::feature_evaluator::{FeatureEvaluator, FeatureExtractor, LinearRegression, Regression};
use gamer::gomoku::{Gomoku, GomokuState, GomokuLineFeatureExtractor};
use gamer::hexapawn::Hexapawn;
use gamer::minimax::{minimax_fixed_depth};
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
    evaluator.train(1000, 0.999, 0.1, &|_, _| ());
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
  let evaluator = TerminalEvaluator::new();

  bench.iter(|| {minimax_fixed_depth(&state, &evaluator, 10, 0.999)});
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
    evaluator.train(1000, 0.999, 0.1, &|_, _| ());
    evaluator
  });
}

fn hexapawn_minimax(bench: &mut Bencher) {
  let game = Hexapawn::new(3, 3);
  let evaluator = TerminalEvaluator::new();
  let state = game.new_game();

  bench.iter(|| {minimax_fixed_depth(&state, &evaluator, 10, 0.999)});
}

fn u32_vec_mult(bench: &mut Bencher) {
  let mut a: Vec<u32> = vec![0; 64];
  let mut b: Vec<u32> = vec![0; 64];

  for i in 0..64 {
    a[i] = i as u32;
    b[i] = 2 * i as u32;
  }

  bench.iter(|| -> u32 { a.iter().zip(b.iter()).map(|(x, y)| (x * y)).sum() });
}

fn f32_vec_mult(bench: &mut Bencher) {
  let mut a: Vec<f32> = vec![0.0; 64];
  let mut b: Vec<f32> = vec![0.0; 64];

  for i in 0..64 {
    a[i] = i as f32;
    b[i] = 2.0 * i as f32;
  }

  bench.iter(|| -> f32 { a.iter().zip(b.iter()).map(|(x, y)| x * y).sum() });
}

fn f64_vec_mult(bench: &mut Bencher) {
  let mut a: Vec<f64> = vec![0.0; 64];
  let mut b: Vec<f64> = vec![0.0; 64];

  for i in 0..64 {
    a[i] = i as f64;
    b[i] = 2.0 * i as f64;
  }

  bench.iter(|| -> f64 { a.iter().zip(b.iter()).map(|(x, y)| x * y).sum() });
}

fn u32_arr_mult(bench: &mut Bencher) {
  let mut a: [u32; 64] = [0; 64];
  let mut b: [u32; 64] = [0; 64];

  for i in 0..64 {
    a[i] = i as u32;
    b[i] = 2 * i as u32;
  }

  bench.iter(|| -> u32 { a.iter().zip(b.iter()).map(|(x, y)| x * y).sum() });
}

fn f32_arr_mult(bench: &mut Bencher) {
  let mut a: [f32; 64] = [0.; 64];
  let mut b: [f32; 64] = [0.; 64];

  for i in 0..64 {
    a[i] = i as f32;
    b[i] = 2.0 * i as f32;
  }

  bench.iter(|| -> f32 { a.iter().zip(b.iter()).map(|(x, y)| x * y).sum() });
}

fn f64_arr_mult(bench: &mut Bencher) {
  let mut a: [f64; 64] = [0.; 64];
  let mut b: [f64; 64] = [0.; 64];

  for i in 0..64 {
    a[i] = i as f64;
    b[i] = 2.0 * i as f64;
  }

  bench.iter(|| -> f64 { a.iter().zip(b.iter()).map(|(x, y)| x * y).sum() });
}

fn xorshift_rng_new(bench: &mut Bencher) {
  bench.iter(|| rand::weak_rng());
}

fn xorshift_rng_new_unseeded(bench: &mut Bencher) {
  bench.iter(|| rand::XorShiftRng::new_unseeded());
}

fn xorshift_rng_new_gen1(bench: &mut Bencher) {
  bench.iter(|| rand::weak_rng().next_u32());
}

fn xorshift_rng_gen1(bench: &mut Bencher) {
  let mut rng = rand::XorShiftRng::new_unseeded();
  bench.iter(|| rng.next_u32());
}

benchmark_group!(benches,
    gomoku_random,
    gomoku_lines_feature_extractor_start,
    gomoku_lines_feature_extractor_rand_position,
    // gomoku_train_evaluator_1000,
    hexapawn_minimax,
    subtractor_random,
    subtractor_minimax,
    subtractor_feature_evaluator,
    subtractor_train_evaluator_1000,
    u32_vec_mult,
    f32_vec_mult,
    f64_vec_mult,
    u32_arr_mult,
    f32_arr_mult,
    f64_arr_mult,
    xorshift_rng_new,
    xorshift_rng_new_unseeded,
    xorshift_rng_new_gen1,
    xorshift_rng_gen1
    );
benchmark_main!(benches);