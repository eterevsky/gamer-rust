use std::time::{Duration, Instant};

use crate::def::{Evaluator, Game, State};
use crate::games::subtractor::{Subtractor, SubtractorState};
use crate::registry::create_training;
use crate::spec::{FeatureExtractorSpec, RegressionSpec, TrainerSpec, TrainingSpec};

fn seconds(d: Duration) -> f64 {
  d.as_secs() as f64 + 1E-9 * d.subsec_nanos() as f64
}

fn select_best_move(
  state: &SubtractorState,
  evaluator: &Evaluator<SubtractorState>,
) -> u32 {
  let current_player = state.player();
  let mut best_move = 0u32;
  let mut best_eval = 0.0;
  let moves = state.iter_moves().collect::<Vec<_>>();
  for m in moves {
    let mut state_copy = state.clone();
    state_copy.play(m).unwrap();
    let eval = evaluator.evaluate_for_player(&state_copy, current_player);
    if best_move == 0 || eval > best_eval {
      best_move = m;
      best_eval = eval;
    }
  }

  best_move
}

pub fn check_evaluator(evaluator: &Evaluator<SubtractorState>) -> i32 {
  let mut correct = 0;
  for i in 1..22 {
    let mut state = Subtractor::new(i, 4).new_game();
    let m = select_best_move(&state, evaluator);
    state.play(m).unwrap();
    if i % 4 == 0 || state.number % 4 == 0 {
      correct += 1;
    }
  }

  return correct;
}

fn evaluate_training(spec: &TrainingSpec) -> (i32, Duration, i32) {
  let game = Subtractor::default(21, 4);
  let mut trainer = create_training(game, spec);

  let start = Instant::now();
  let mut steps = 0;
  let mut correct = 0;
  let mut last_check = Instant::now() - Duration::from_secs(1);
  while Instant::now() - start < Duration::from_secs(5) {
    trainer.train(1, Duration::new(0, 0));
    steps += 1;

    if Instant::now() - last_check > Duration::from_millis(1) {
      let evaluator: Box<Evaluator<SubtractorState>> =
        trainer.build_evaluator();
      correct = check_evaluator(evaluator.as_ref());
      if correct == 21 {
        break;
      }
      last_check = Instant::now()
    }
  }

  println!(
    "{:?}\nSteps: {}\nTime: {:?}\nCorrect {}\n",
    spec,
    steps,
    Instant::now() - start,
    correct
  );

  (21 - correct, Instant::now() - start, steps)
}

pub fn train_subtractor_eval() {
  let mut training_spec = TrainingSpec {
    extractor: FeatureExtractorSpec::Subtractor(10),
    regression: RegressionSpec {
      params: vec![],
      regularization: 0.001,
    },
    trainer: TrainerSpec::Reinforce {
      minimax_depth: 5,
      random_prob: 0.1,
      alpha: 0.001,
    },
  };

  let minimax_values = vec![1, 2, 3, 4];
  // let random_prob_values = vec![0.01, 0.03, 0.05, 0.1, 0.3, 0.5];
  // let alpha_values = vec![0.00001, 0.0001, 0.001, 0.01, 0.1, 1.0];

  // let mut configs = vec![];

  // for &minimax_depth in minimax_values.iter() {
  //   for &random_prob in random_prob_values.iter() {
  //     for &alpha in alpha_values.iter() {
  //       training_spec.trainer = TrainerSpec::Reinforce {
  //         minimax_depth,
  //         random_prob,
  //         alpha,
  //       };

  //       let (wrong, t, steps) = evaluate_training(&training_spec);
  //       configs.push((minimax_depth, random_prob, alpha, wrong, t, steps))
  //     }
  //   }
  // }

  // configs.sort_unstable_by_key(|c| (c.3, c.4));
  // for c in configs {
  //   let (depth, random_prob, alpha, wrong, t, steps) = c;
  //   println!("depth = {}  random = {}  alpha = {}  ->  wrong = {}  time = {:.3}  steps = {}",
  //        depth, random_prob, alpha, wrong, seconds(t), steps);;
  // }

  let mut configs = vec![];

  let step_sizes = vec![0.01, 0.1, 1.0];
  let temperatures = vec![1.0, 10.0, 100.0];
  let ngameses = vec![1, 3, 5];

  for &minimax_depth in minimax_values.iter() {
    for &step_size in step_sizes.iter() {
      for &temperature in temperatures.iter() {
        for &ngames in ngameses.iter() {
          training_spec.trainer = TrainerSpec::Annealing {
            step_size,
            minimax_depth,
            temperature,
            ngames
          };

          let (wrong, t, steps) = evaluate_training(&training_spec);
          configs.push((minimax_depth, step_size, temperature, ngames, wrong, t, steps))
        }
      }
    }
  }

  configs.sort_unstable_by_key(|c| (c.4, c.5));
  for c in configs {
    let (minimax_depth, step_size, temperature, ngames, wrong, t, steps) = c;
    println!("depth = {}  step = {}  temp = {}  ngames = {}  ->  wrong = {}  time = {:.3}  steps = {}",
         minimax_depth, step_size, temperature, ngames, wrong, seconds(t), steps);;
  }
}
