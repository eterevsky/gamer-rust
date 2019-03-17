use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Read;
use std::time::Duration;

#[derive(Debug)]
pub enum GameSpec {
  Gomoku,
  Hexapawn(u32, u32),
  Subtractor(u32, u32),
}

impl GameSpec {
  pub fn parse(s: &str) -> Option<GameSpec> {
    match s {
      "gomoku" => Some(GameSpec::Gomoku),
      "hexapawn" => Some(GameSpec::Hexapawn(8, 8)),
      "subtractor" => Some(GameSpec::Subtractor(21, 4)),
      _ => None,
    }
  }
}

#[derive(Clone, Serialize, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum AgentSpec {
  Random,
  Human,
  Minimax {
    depth: u32,
    #[serde(default)]
    time_per_move: f64,
    evaluator: EvaluatorSpec,
    #[serde(default)]
    name: String,
  },
  Mcts {
    policy: PolicySpec,
    evaluator: EvaluatorSpec,
    #[serde(default)]
    samples: u64,
    #[serde(default)]
    time_per_move: f64,
    #[serde(default)]
    name: String,
  },
}

#[derive(Clone, Serialize, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum EvaluatorSpec {
  Terminal,
  Features {
    extractor: FeatureExtractorSpec,
    regression: RegressionSpec,
  },
  Sampler {
    samples: usize,
    discount: f32,
  },
}

#[derive(Clone, Serialize, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum PolicySpec {
  Equal,
}

#[derive(Clone, Serialize, Debug, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum FeatureExtractorSpec {
  Subtractor(u32),
  GomokuLines(u32),
  HexapawnNumberOfPawns,
  HexapawnComplete,
}

#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct RegressionSpec {
  pub params: Vec<f32>,
  pub regularization: f32,
}

#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct TrainingSpec {
  pub extractor: FeatureExtractorSpec,
  pub regression: RegressionSpec,
  pub trainer: TrainerSpec,
}

#[derive(Clone, Serialize, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum TrainerSpec {
  Reinforce {
    minimax_depth: u32,
    random_prob: f32,
    alpha: f32,
  },
  Annealing {
    step_size: f32,
    minimax_depth: u32,
    temperature: f32, // For each of this number of steps the temperature will fall exponentially.
    ngames: u32, // Number of games between players to determine the winner.
  },
  LadderAnnealing {
    step_size: f32,
    minimax_depth: u32,
    temperature: f32,
    ngames: usize,
  },
}

/// First match the string with "random" or "human". If it doesn't, treat it
/// as a filename. One the file, read the contents as JSON and convert to
/// AgentSpec.
pub fn load_agent_spec(
  fname: &str,
  time_per_move: Duration,
) -> Result<AgentSpec, String> {
  let time_per_move =
    time_per_move.as_secs() as f64 + time_per_move.subsec_nanos() as f64 * 1E-9;
  match fname {
    "random" => Ok(AgentSpec::Random),

    "human" => Ok(AgentSpec::Human),

    _ => {
      // Treating the string as filename.
      let mut f = File::open(fname)
        .map_err(|e| format!("Error while opening file: {}", e))?;
      let mut s = String::new();
      f.read_to_string(&mut s)
        .map_err(|e| format!("Error while reading file: {}", e))?;
      let mut spec = serde_json::from_str(&s)
        .map_err(|e| format!("Error while parsing AgentSpec: {}", e))?;

      match spec {
        AgentSpec::Minimax {
          depth: _,
          time_per_move: ref mut t,
          evaluator: _,
          ref mut name,
        } => {
          if time_per_move > 0.0 {
            *t = time_per_move
          }
          if name.is_empty() {
            *name = fname.to_string()
          }
        }
        AgentSpec::Mcts {
          policy: _,
          evaluator: _,
          samples: _,
          time_per_move: ref mut t,
          ref mut name,
        } => {
          if time_per_move > 0.0 {
            *t = time_per_move
          }
          if name.is_empty() {
            *name = fname.to_string()
          }
        }
        _ => (),
      }

      Ok(spec)
    }
  }
}

pub fn load_evaluator_spec(s: &str) -> Result<EvaluatorSpec, String> {
  let mut f =
    File::open(s).map_err(|e| format!("Error while opening file: {}", e))?;
  let mut s = String::new();
  f.read_to_string(&mut s)
    .map_err(|e| format!("Error while reading file: {}", e))?;
  serde_json::from_str(&s)
    .map_err(|e| format!("Error while parsing EvaluatorSpec: {}", e))
}

pub fn load_training_spec(s: &str) -> Result<TrainingSpec, String> {
  let mut f =
    File::open(s).map_err(|e| format!("Error while opening file: {}", e))?;
  let mut s = String::new();
  f.read_to_string(&mut s)
    .map_err(|e| format!("Error while reading file: {}", e))?;
  serde_json::from_str(&s)
    .map_err(|e| format!("Error while parsing TrainingSpec: {}", e))
}

pub fn agent_spec_to_json(agent_spec: &AgentSpec) -> String {
  serde_json::to_string_pretty(&agent_spec).unwrap()
}

#[cfg(test)]
mod test {

  use serde_json;

  use super::*;

  #[test]
  fn to_json_from_json() {
    let agent_spec = AgentSpec::Minimax {
      depth: 3,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::Features {
        extractor: FeatureExtractorSpec::Subtractor(10),
        regression: RegressionSpec {
          params: vec![0.1, 0.2, 0.3],
          regularization: 0.001,
        },
      },
      name: String::new(),
    };

    let agent_json = serde_json::to_string_pretty(&agent_spec).unwrap();
    println!("{}", agent_json);
    let agent_spec2: AgentSpec = serde_json::from_str(&agent_json).unwrap();
    let agent_json2 = serde_json::to_string_pretty(&agent_spec2).unwrap();

    assert_eq!(agent_json, agent_json2);
  }

  #[test]
  fn terminal_evaluator() {
    let agent_spec = AgentSpec::Minimax {
      depth: 3,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::Terminal,
      name: String::new(),
    };

    let agent_json = serde_json::to_string_pretty(&agent_spec).unwrap();
    println!("{}", agent_json);
  }

} // mod test
