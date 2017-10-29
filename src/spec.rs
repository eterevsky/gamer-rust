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
    #[serde(default)] time_per_move: f64,
    evaluator: EvaluatorSpec,
  },
}

#[derive(Clone, Serialize, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum EvaluatorSpec {
  Terminal,
  Features {
    extractor: FeatureExtractorSpec,
    regression: RegressionSpec,
    training_minimax_depth: u32,
    #[serde(default)] steps: u64,
  },
  Sampler {
    nsamples: usize,
    discount: f32,
  }
}

#[derive(Clone, Serialize, Debug, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum FeatureExtractorSpec {
  Subtractor(u32),
  GomokuLines,
  HexapawnNumberOfPawns,
}

#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct RegressionSpec {
  pub speed: f32,
  pub regularization: f32,
  pub b: Vec<f32>,
}

/// First match the string with "random" or "human". If it doesn't, treat it
/// as a filename. One the file, read the contents as JSON and convert to
/// AgentSpec.
pub fn load_agent_spec(
  s: &str,
  time_per_move: Duration,
) -> Result<AgentSpec, String> {
  let time_per_move =
    time_per_move.as_secs() as f64 + time_per_move.subsec_nanos() as f64 * 1E-9;
  match s {
    "random" => Ok(AgentSpec::Random),

    "human" => Ok(AgentSpec::Human),

    _ => {
      // Treating the string as filename.
      let mut f =
        File::open(s).map_err(|e| format!("Error while opening file: {}", e))?;
      let mut s = String::new();
      f.read_to_string(&mut s)
        .map_err(|e| format!("Error while reading file: {}", e))?;
      let mut spec = serde_json::from_str(&s)
        .map_err(|e| format!("Error while parsing AgentSpec: {}", e))?;
      if let AgentSpec::Minimax {
        depth: _,
        time_per_move: ref mut t,
        evaluator: _,
      } = spec
      {
        if time_per_move > 0.0 {
          *t = time_per_move
        }
      };
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
          speed: 0.001,
          regularization: 0.001,
          b: vec![0.1, 0.2, 0.3],
        },
        training_minimax_depth: 1,
        steps: 0,
      },
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
    };

    let agent_json = serde_json::to_string_pretty(&agent_spec).unwrap();
    println!("{}", agent_json);
  }

} // mod test
