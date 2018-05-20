#[macro_use]
extern crate bencher;
extern crate rand;

extern crate gamer;

use bencher::Bencher;
use rand::Rng;
use std::time::Duration;

use gamer::def::{Evaluator, FeatureExtractor, Game, State};
use gamer::evaluators::{FeatureEvaluator, LinearRegressionTanh, TerminalEvaluator};
use gamer::games::{Gomoku, GomokuLineFeatureExtractor, Hexapawn, Subtractor, SubtractorFeatureExtractor};
use gamer::agents::minimax_fixed_depth;
use gamer::spec::{FeatureExtractorSpec, TrainerSpec, TrainingSpec, RegressionSpec};
use gamer::registry::create_training;

fn generate_random_gomoku_position() -> <Gomoku as Game>::State {
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
    state.payoff().unwrap()
  })
}

fn gomoku_lines_feature_extractor_start(bench: &mut Bencher) {
  let state = Gomoku::default().new_game();
  let feature_extractor = GomokuLineFeatureExtractor::default(2);
  bench.iter(|| {feature_extractor.extract(&state)});
}

fn gomoku_lines_feature_extractor_rand_position(bench: &mut Bencher) {
  let feature_extractor = GomokuLineFeatureExtractor::default(2);
  let state = generate_random_gomoku_position();
  bench.iter(|| {feature_extractor.extract(&state)});
}

// fn gomoku_train_evaluator_1000(bench: &mut Bencher) {
//   let game = Gomoku::default();

//   bench.iter(|| {
//     let extractor = GomokuLineFeatureExtractor::new(2);
//     let regression = LinearRegressionTanh::new(&[0.0; 33], 0.001);
//     let mut evaluator = FeatureEvaluator::new(game, extractor, regression, 1, 0);
//     evaluator.train(1000, Duration::new(0, 0));
//     evaluator
//   });
// }

fn subtractor_random(bench: &mut Bencher) {
  let game = Subtractor::new(21, 4);
  let mut rng = rand::weak_rng();
  bench.iter(|| {
    let mut state = game.new_game();
    while let Some(m) = state.get_random_move(&mut rng) {
      state.play(m).unwrap();
    }
    state.payoff().unwrap()
  })
}

fn subtractor_minimax(bench: &mut Bencher) {
  let state = Subtractor::new(21, 4).new_game();
  let evaluator = TerminalEvaluator::new();

  bench.iter(|| {minimax_fixed_depth(&state, &evaluator, 10, 0.999)});
}

fn subtractor_feature_evaluator(bench: &mut Bencher) {
  let game = Subtractor::default(21, 4);
  let extractor = SubtractorFeatureExtractor::new(10);
  let regression = LinearRegressionTanh::new(
      &[0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.0);
  let evaluator = FeatureEvaluator::new(game, extractor, regression);

  let state = game.new_game();
  bench.iter(|| {evaluator.evaluate(&state)});
}

fn subtractor_train_evaluator_1000(bench: &mut Bencher) {
  let game = Subtractor::default(21, 4);
  let training_spec = TrainingSpec {
    extractor: FeatureExtractorSpec::Subtractor(10),
    regression: RegressionSpec {
      params: vec![],
      regularization: 0.001,
    },
    trainer: TrainerSpec::Reinforce {
      minimax_depth: 1,
      random_prob: 0.1,
      alpha: 0.001,
    },
  };

  let mut trainer = create_training(game, &training_spec);

  bench.iter(|| {
    trainer.train(1000, Duration::new(0, 0));
    trainer.build_evaluator()
  });
}

fn hexapawn_minimax(bench: &mut Bencher) {
  let game = Hexapawn::new(3, 3);
  let evaluator = TerminalEvaluator::new();
  let state = game.new_game();

  bench.iter(|| {minimax_fixed_depth(&state, &evaluator, 10, 0.999)});
}

fn init_vecs_u32() -> (Vec<u32>, Vec<u32>) {
  let mut a = vec![0; 64];
  let mut b = vec![0; 64];

  for i in 0..64 {
    a[i] = i as u32;
    b[i] = (2 * i) as u32;
  }

  (a, b)
}

fn init_vecs_f32() -> (Vec<f32>, Vec<f32>) {
  let mut a = vec![0.0; 64];
  let mut b = vec![0.0; 64];

  for i in 0..64 {
    a[i] = i as f32;
    b[i] = (2 * i) as f32;
  }

  (a, b)
}

fn init_vecs_f64() -> (Vec<f64>, Vec<f64>) {
  let mut a = vec![0.0; 64];
  let mut b = vec![0.0; 64];

  for i in 0..64 {
    a[i] = i as f64;
    b[i] = (2 * i) as f64;
  }

  (a, b)
}

fn f32_vec_mult(bench: &mut Bencher) {
  let (a, b) = init_vecs_f32();
  let (xs, ys): (&[f32], &[f32]) = (&a, &b);

  bench.iter(|| -> f32 { xs.iter().zip(ys.iter()).map(|(x, y)| x * y).sum() });
}

fn f64_vec_mult(bench: &mut Bencher) {
  let (a, b) = init_vecs_f64();
  let (xs, ys): (&[f64], &[f64]) = (&a, &b);

  bench.iter(|| -> f64 { xs.iter().zip(ys.iter()).map(|(x, y)| x * y).sum() });
}

fn f32_vec_mult_par2(bench: &mut Bencher) {
  let (a, b) = init_vecs_f32();
  let (xs, ys): (&[f32], &[f32]) = (&a, &b);

  bench.iter(|| -> f32 {
    let (mut s0, mut s1) = (0., 0.);
    let (mut xs, mut ys) = (xs, ys);
    while xs.len() >= 2 {
      s0 += xs[0] * ys[0];
      s1 += xs[1] * ys[1];
      xs = &xs[2..];
      ys = &ys[2..];
    }
    s0 + s1
  });
}

fn f64_vec_mult_par2(bench: &mut Bencher) {
  let (a, b) = init_vecs_f64();
  let (xs, ys): (&[f64], &[f64]) = (&a, &b);

  bench.iter(|| -> f64 {
    let (mut s0, mut s1) = (0., 0.);
    let (mut xs, mut ys) = (xs, ys);
    while xs.len() >= 2 {
      s0 += xs[0] * ys[0];
      s1 += xs[1] * ys[1];
      xs = &xs[2..];
      ys = &ys[2..];
    }
    s0 + s1
  });
}

fn f32_vec_mult_par4(bench: &mut Bencher) {
  let (a, b) = init_vecs_f32();
  let (xs, ys): (&[f32], &[f32]) = (&a, &b);

  bench.iter(|| -> f32 {
    let (mut s0, mut s1, mut s2, mut s3) = (0., 0., 0., 0.);
    let (mut xs, mut ys) = (xs, ys);
    while xs.len() >= 4 {
      s0 += xs[0] * ys[0];
      s1 += xs[1] * ys[1];
      s2 += xs[2] * ys[2];
      s3 += xs[3] * ys[3];
      xs = &xs[4..];
      ys = &ys[4..];
    }
    s0 + s1 + s2 + s3
  });
}

fn f32_vec_mult_par4_2(bench: &mut Bencher) {
  let (a, b) = init_vecs_f32();
  let (xs, ys): (&[f32], &[f32]) = (&a, &b);

  bench.iter(|| -> f32 { xs.chunks(4).zip(ys.chunks(4)).map(|(xx, yy)| {
    unsafe {
      xx.get_unchecked(0) * yy.get_unchecked(0) +
      xx.get_unchecked(1) * yy.get_unchecked(1) +
      xx.get_unchecked(2) * yy.get_unchecked(2) +
      xx.get_unchecked(3) * yy.get_unchecked(3)
    }
  }).sum() });
}

fn f32_vec_mult_par4_3(bench: &mut Bencher) {
  let (a, b) = init_vecs_f32();
  let (xs, ys): (&[f32], &[f32]) = (&a, &b);

  bench.iter(|| -> f32 { xs.chunks(4).zip(ys.chunks(4)).map(|(xx, yy)| {
    xx[0] * yy[0] + xx[1] * yy[1] + xx[2] * yy[2] + xx[3] * yy[3]
  }).sum() });
}

fn f32_vec_mult_par4_4(bench: &mut Bencher) {
  let (a, b) = init_vecs_f32();
  let (xs, ys): (&[f32], &[f32]) = (&a, &b);

  bench.iter(|| -> f32 {
    let (mut s0, mut s1, mut s2, mut s3) = (0., 0., 0., 0.);
    let (mut xs, mut ys) = (xs, ys);
    while xs.len() >= 4 {
      unsafe {
        s0 += xs.get_unchecked(0) * ys.get_unchecked(0);
        s1 += xs.get_unchecked(1) * ys.get_unchecked(1);
        s2 += xs.get_unchecked(2) * ys.get_unchecked(2);
        s3 += xs.get_unchecked(3) * ys.get_unchecked(3);
        xs = &xs[4..];
        ys = &ys[4..];
      }
    }
    s0 + s1 + s2 + s3
  });
}

// fn f32_vec_mult_par4_simd(bench: &mut Bencher) {
//   let (a, b): (Vec<f32>, Vec<f32>) = init_vecs_f32();
//   let aslice = a.as_slice();
//   let bslice = b.as_slice();

//   bench.iter(|| -> f32 {
//     let mut partial_sum = simd::f32x4::splat(0.0);
//     let mut i = 0;
//     while i < a.len() {
//       let xs = simd::f32x4::load(aslice, i);
//       let ys = simd::f32x4::load(bslice, i);
//       partial_sum = partial_sum + xs * ys;
//       i += 4;
//     }
//     partial_sum.extract(0) + partial_sum.extract(1) + partial_sum.extract(2) + partial_sum.extract(3)
//   });
// }

fn f64_vec_mult_par4(bench: &mut Bencher) {
  let (a, b) = init_vecs_f64();
  let (xs, ys): (&[f64], &[f64]) = (&a, &b);

  bench.iter(|| -> f64 {
    let (mut s0, mut s1, mut s2, mut s3) = (0., 0., 0., 0.);
    let (mut xs, mut ys) = (xs, ys);
    while xs.len() >= 4 {
      s0 += xs[0] * ys[0];
      s1 += xs[1] * ys[1];
      s2 += xs[2] * ys[2];
      s3 += xs[3] * ys[3];
      xs = &xs[4..];
      ys = &ys[4..];
    }
    s0 + s1 + s2 + s3
  });
}

fn f32_vec_mult_par8(bench: &mut Bencher) {
  let (a, b) = init_vecs_f32();
  let (xs, ys): (&[f32], &[f32]) = (&a, &b);

  bench.iter(|| -> f32 {
    let (mut s0, mut s1, mut s2, mut s3, mut s4, mut s5, mut s6, mut s7) =
        (0., 0., 0., 0., 0., 0., 0., 0.);
    let (mut xs, mut ys) = (xs, ys);
    while xs.len() >= 4 {
      s0 += xs[0] * ys[0];
      s1 += xs[1] * ys[1];
      s2 += xs[2] * ys[2];
      s3 += xs[3] * ys[3];
      s4 += xs[4] * ys[4];
      s5 += xs[5] * ys[5];
      s6 += xs[6] * ys[6];
      s7 += xs[7] * ys[7];
      xs = &xs[8..];
      ys = &ys[8..];
    }
    s0 + s1 + s2 + s3 + s4 + s5 + s6 + s7
  });
}

fn f64_vec_mult_par8(bench: &mut Bencher) {
  let (a, b) = init_vecs_f64();
  let (xs, ys): (&[f64], &[f64]) = (&a, &b);

  bench.iter(|| -> f64 {
    let (mut s0, mut s1, mut s2, mut s3, mut s4, mut s5, mut s6, mut s7) =
        (0., 0., 0., 0., 0., 0., 0., 0.);
    let (mut xs, mut ys) = (xs, ys);
    while xs.len() >= 4 {
      s0 += xs[0] * ys[0];
      s1 += xs[1] * ys[1];
      s2 += xs[2] * ys[2];
      s3 += xs[3] * ys[3];
      s4 += xs[4] * ys[4];
      s5 += xs[5] * ys[5];
      s6 += xs[6] * ys[6];
      s7 += xs[7] * ys[7];
      xs = &xs[8..];
      ys = &ys[8..];
    }
    s0 + s1 + s2 + s3 + s4 + s5 + s6 + s7
  });
}

// Yay!
fn f32_vec_mult_par8_2(bench: &mut Bencher) {
  let (a, b) = init_vecs_f32();
  let (xs, ys): (&[f32], &[f32]) = (&a, &b);

  bench.iter(|| -> f32 {
    let (mut s0, mut s1, mut s2, mut s3, mut s4, mut s5, mut s6, mut s7) =
        (0., 0., 0., 0., 0., 0., 0., 0.);
    let (mut xs, mut ys) = (xs, ys);
    while xs.len() >= 8 {
      unsafe {
        s0 += xs.get_unchecked(0) * ys.get_unchecked(0);
        s1 += xs.get_unchecked(1) * ys.get_unchecked(1);
        s2 += xs.get_unchecked(2) * ys.get_unchecked(2);
        s3 += xs.get_unchecked(3) * ys.get_unchecked(3);
        s4 += xs.get_unchecked(4) * ys.get_unchecked(4);
        s5 += xs.get_unchecked(5) * ys.get_unchecked(5);
        s6 += xs.get_unchecked(6) * ys.get_unchecked(6);
        s7 += xs.get_unchecked(7) * ys.get_unchecked(7);
        xs = &xs[8..];
        ys = &ys[8..];
      }
    }
    s0 + s1 + s2 + s3 + s4 + s5 + s6 + s7
  });
}

fn f64_vec_mult_par8_2(bench: &mut Bencher) {
  let (a, b) = init_vecs_f64();
  let (xs, ys): (&[f64], &[f64]) = (&a, &b);

  bench.iter(|| -> f64 {
    let (mut s0, mut s1, mut s2, mut s3, mut s4, mut s5, mut s6, mut s7) =
        (0., 0., 0., 0., 0., 0., 0., 0.);
    let (mut xs, mut ys) = (xs, ys);
    while xs.len() >= 8 {
      unsafe {
        s0 += xs.get_unchecked(0) * ys.get_unchecked(0);
        s1 += xs.get_unchecked(1) * ys.get_unchecked(1);
        s2 += xs.get_unchecked(2) * ys.get_unchecked(2);
        s3 += xs.get_unchecked(3) * ys.get_unchecked(3);
        s4 += xs.get_unchecked(4) * ys.get_unchecked(4);
        s5 += xs.get_unchecked(5) * ys.get_unchecked(5);
        s6 += xs.get_unchecked(6) * ys.get_unchecked(6);
        s7 += xs.get_unchecked(7) * ys.get_unchecked(7);
        xs = &xs[8..];
        ys = &ys[8..];
      }
    }
    s0 + s1 + s2 + s3 + s4 + s5 + s6 + s7
  });
}

fn u32_vec_mult_par8_2(bench: &mut Bencher) {
  let (a, b) = init_vecs_u32();
  let (xs, ys): (&[u32], &[u32]) = (&a, &b);

  bench.iter(|| -> u32 {
    let (mut s0, mut s1, mut s2, mut s3, mut s4, mut s5, mut s6, mut s7) =
        (0, 0, 0, 0, 0, 0, 0, 0);
    let (mut xs, mut ys) = (xs, ys);
    while xs.len() >= 8 {
      unsafe {
        s0 += xs.get_unchecked(0) * ys.get_unchecked(0);
        s1 += xs.get_unchecked(1) * ys.get_unchecked(1);
        s2 += xs.get_unchecked(2) * ys.get_unchecked(2);
        s3 += xs.get_unchecked(3) * ys.get_unchecked(3);
        s4 += xs.get_unchecked(4) * ys.get_unchecked(4);
        s5 += xs.get_unchecked(5) * ys.get_unchecked(5);
        s6 += xs.get_unchecked(6) * ys.get_unchecked(6);
        s7 += xs.get_unchecked(7) * ys.get_unchecked(7);
        xs = &xs[8..];
        ys = &ys[8..];
      }
    }
    s0 + s1 + s2 + s3 + s4 + s5 + s6 + s7
  });
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
    f32_arr_mult,
    f32_vec_mult,
    f32_vec_mult_par2,
    f32_vec_mult_par4,
    f32_vec_mult_par4_2,
    f32_vec_mult_par4_3,
    f32_vec_mult_par4_4,
    f32_vec_mult_par8,
    f32_vec_mult_par8_2,
    f64_arr_mult,
    f64_vec_mult,
    f64_vec_mult_par2,
    f64_vec_mult_par4,
    f64_vec_mult_par8,
    f64_vec_mult_par8_2,
    u32_vec_mult_par8_2,
    gomoku_lines_feature_extractor_rand_position,
    gomoku_lines_feature_extractor_start,
    gomoku_random,
    // gomoku_train_evaluator_1000,
    hexapawn_minimax,
    subtractor_random,
    subtractor_minimax,
    subtractor_feature_evaluator,
    subtractor_train_evaluator_1000,
    xorshift_rng_new,
    xorshift_rng_new_unseeded,
    xorshift_rng_new_gen1,
    xorshift_rng_gen1
    );
benchmark_main!(benches);