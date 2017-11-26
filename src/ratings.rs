use std::f32;
use std::fmt;
use std::fmt::Write;

use ladder::GameResult;
use opt::minimize;

pub struct Ratings {
  k: f64,   // normalization of logistic function
  reg: f64, // regularization coefficient
  ratings: Vec<f32>,
  played_games: Vec<u32>,
  results: Vec<GameResult>,
  min_rating: f32,
}

impl Ratings {
  pub fn new() -> Self {
    Ratings {
      k: 3.0f64.ln() / 400.,
      reg: 1E-6,
      ratings: vec![],
      played_games: vec![],
      results: vec![],
      min_rating: 0.0,
    }
  }

  pub fn add_game(&mut self, result: GameResult) {
    self.results.push(result);

    while self.ratings.len() <= result.player1_id
      || self.ratings.len() <= result.player2_id
    {
      self.ratings.push(0.0);
      self.played_games.push(0);
    }

    self.played_games[result.player1_id] += 1;
    self.played_games[result.player2_id] += 1;
  }

  pub fn full_update(&mut self) {
    let mut ratings = self.ratings.clone();
    minimize(
      &|r| self.log_prob(r),
      &|r| self.log_prob_grad(r),
      ratings.as_mut_slice(),
    );
    self.ratings = ratings;
    self.min_rating = f32::MAX;
    for &r in self.ratings.iter() {
      if r < self.min_rating {
        self.min_rating = r;
      }
    }
  }

  fn logistic_function(&self, diff: f64) -> f64 {
    0.5 * (diff * self.k).tanh() + 0.5
  }

  fn logistic_derivative(&self, diff: f64) -> f64 {
    let tanh = (self.k * diff).tanh();
    0.5 * self.k * (1. - tanh * tanh)
  }

  fn log_prob(&self, ratings: &[f32]) -> f32 {
    let mut sum = 0f64;
    for result in self.results.iter() {
      let diff =
        ratings[result.player1_id] as f64 - ratings[result.player2_id] as f64;
      let prob = self.logistic_function(diff);
      sum -= 0.5 * (result.payoff as f64 + 1.0) * prob.ln()
        + 0.5 * (result.payoff as f64 - 1.0) * (1. - prob).ln();
    }

    for &r in ratings {
      let r = r as f64;
      sum += self.reg * r * r;
    }

    sum as f32
  }

  fn log_prob_grad(&self, ratings: &[f32]) -> Vec<f32> {
    let mut grad = Vec::with_capacity(ratings.len());
    for &r in ratings {
      grad.push(2. * self.reg * r as f64);
    }

    for result in self.results.iter() {
      let diff =
        ratings[result.player1_id] as f64 - ratings[result.player2_id] as f64;
      let prob = self.logistic_function(diff);
      let deriv = self.logistic_derivative(diff);
      let d = 0.5 * deriv
        * ((result.payoff as f64 + 1.) / prob
          + (result.payoff as f64 - 1.) / (1. - prob));
      grad[result.player1_id] -= d;
      grad[result.player2_id] += d;
    }

    grad.iter().map(|&x| x as f32).collect::<Vec<_>>()
  }

  pub fn get_rating(&self, player_id: usize) -> f32 {
    self.ratings[player_id] - self.ratings[0]
  }

  pub fn print<'a>(&self, names: Vec<&'a str>) -> String {
    let mut indices: Vec<_> = (0..self.ratings.len()).collect();
    indices.sort_unstable_by(|&i, &j| {
      self.ratings[j].partial_cmp(&self.ratings[i]).unwrap()
    });
    let mut s = String::new();
    for i in indices {
      writeln!(
        &mut s,
        "{:32}  {:>6.1}  {:>2}",
        names[i],
        self.ratings[i] - self.min_rating,
        self.played_games[i]
      ).unwrap();
    }
    s
  }

  #[cfg(test)]
  fn predict_payoff(&self, player1_id: usize, player2_id: usize) -> f32 {
    let diff =
      self.ratings[player1_id] as f64 - self.ratings[player2_id] as f64;
    (2.0 * self.logistic_function(diff) - 1.0) as f32
  }
}

impl fmt::Display for Ratings {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let mut indices: Vec<_> = (0..self.ratings.len()).collect();
    indices.sort_unstable_by(|&i, &j| {
      self.ratings[j].partial_cmp(&self.ratings[i]).unwrap()
    });
    for i in indices {
      writeln!(
        f,
        "{}  {:.1}  {}",
        i,
        self.ratings[i] - self.min_rating,
        self.played_games[i]
      )?;
    }
    Ok(())
  }
}


#[cfg(test)]
mod test {

  use super::*;
  use ladder::GameResult;

  fn approx_derivative(f: &Fn(f64) -> f64, x: f64) -> f64 {
    let eps = 1E-3 * x.abs();
    let eps = if eps < 1E-3 { 1E-3 } else { eps };
    println!("{} {}", x + eps, f(x + eps));
    println!("{} {}", x - eps, f(x - eps));
    ((f(x + eps) - f(x - eps)) / (2. * eps))
  }

  #[test]
  fn logistic_derivative() {
    let ratings = Ratings::new();
    assert_relative_eq!(
      approx_derivative(&|x| ratings.logistic_function(x), 0.),
      ratings.logistic_derivative(0.),
      max_relative = 1E-4
    );
    assert_relative_eq!(
      approx_derivative(&|x| ratings.logistic_function(x), 200.),
      ratings.logistic_derivative(200.),
      max_relative = 1E-4
    );
    assert_relative_eq!(
      approx_derivative(&|x| ratings.logistic_function(x), -200.),
      ratings.logistic_derivative(-200.),
      max_relative = 1E-4
    );
    assert_relative_eq!(
      approx_derivative(&|x| ratings.logistic_function(x), 456.),
      ratings.logistic_derivative(456.),
      max_relative = 1E-4
    );
    assert_relative_eq!(
      approx_derivative(&|x| ratings.logistic_function(x), -456.),
      ratings.logistic_derivative(-456.),
      max_relative = 1E-4
    );
  }

  #[test]
  fn regularization() {
    let mut ratings = Ratings::new();
    ratings.played_games.push(0);
    ratings.played_games.push(0);
    ratings.ratings.push(0.0);
    ratings.ratings.push(1000.0);

    ratings.full_update();

    assert_eq!(0.0, ratings.ratings[0]);
    assert!(ratings.ratings[1] < 1.0);
  }

  #[test]
  fn single_game() {
    let mut ratings = Ratings::new();
    ratings.add_game(GameResult {
      player1_id: 0,
      player2_id: 1,
      payoff: 1.0,
    });

    ratings.full_update();

    assert!(ratings.ratings[0] - ratings.ratings[1] > 200.0);
    assert!(ratings.predict_payoff(0, 1) > 0.5);
    assert!(ratings.predict_payoff(1, 0) < -0.5);

    println!("{}", ratings);
    println!("{}", ratings.predict_payoff(0, 1));
  }

  #[test]
  fn two_to_one() {
    let mut ratings = Ratings::new();
    ratings.add_game(GameResult {
      player1_id: 0,
      player2_id: 1,
      payoff: 1.0,
    });
    ratings.add_game(GameResult {
      player1_id: 0,
      player2_id: 1,
      payoff: 1.0,
    });
    ratings.add_game(GameResult {
      player1_id: 0,
      player2_id: 1,
      payoff: -1.0,
    });

    ratings.full_update();

    println!("{}", ratings);
    println!("{}", ratings.predict_payoff(0, 1));

    assert!((ratings.predict_payoff(0, 1) - 1. / 3.).abs() < 0.1);
  }

  #[test]
  fn three_to_one() {
    let mut ratings = Ratings::new();
    ratings.add_game(GameResult {
      player1_id: 0,
      player2_id: 1,
      payoff: 1.0,
    });
    ratings.add_game(GameResult {
      player1_id: 0,
      player2_id: 1,
      payoff: 1.0,
    });
    ratings.add_game(GameResult {
      player1_id: 0,
      player2_id: 1,
      payoff: 1.0,
    });
    ratings.add_game(GameResult {
      player1_id: 0,
      player2_id: 1,
      payoff: -1.0,
    });

    ratings.full_update();

    println!("{}", ratings);
    println!("{}", ratings.predict_payoff(0, 1));

    assert_relative_eq!(
      200.0,
      ratings.ratings[0] - ratings.ratings[1],
      max_relative = 0.1
    );
  }

  #[test]
  fn nine_to_one() {
    let mut ratings = Ratings::new();
    for _ in 0..9 {
      ratings.add_game(GameResult {
        player1_id: 0,
        player2_id: 1,
        payoff: 1.0,
      });
    }
    ratings.add_game(GameResult {
      player1_id: 0,
      player2_id: 1,
      payoff: -1.0,
    });

    ratings.full_update();

    println!("{}", ratings);
    println!("{}", ratings.predict_payoff(0, 1));

    assert!((ratings.predict_payoff(0, 1) - 0.8).abs() < 0.1);
  }

  #[test]
  fn three_games() {
    let mut ratings = Ratings::new();
    ratings.add_game(GameResult {
      player1_id: 0,
      player2_id: 1,
      payoff: -1.0,
    });
    ratings.add_game(GameResult {
      player1_id: 1,
      player2_id: 2,
      payoff: 1.0,
    });
    ratings.add_game(GameResult {
      player1_id: 2,
      player2_id: 0,
      payoff: -1.0,
    });

    ratings.full_update();

    println!("{}", ratings);

    assert!(ratings.get_rating(1) - ratings.get_rating(0) > 200.0);
    assert!(ratings.get_rating(0) - ratings.get_rating(2) > 200.0);
  }

} // mod tests
