use serde_json;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub enum GameSpec {
  Gomoku,
  Hexapawn(u32, u32),
  Subtractor(u32, u32)
}

impl GameSpec {
  pub fn parse(s: &str) -> Option<GameSpec> {
    match s {
      "gomoku" => Some(GameSpec::Gomoku),
      "hexapawn" => Some(GameSpec::Hexapawn(8, 8)),
      "subtractor" => Some(GameSpec::Subtractor(21, 4)),
      _ => None
    }
  }
}


#[derive(Serialize, Debug, Deserialize)]
pub enum AgentSpec {
  Random,
  Human,
  Minimax(MinimaxSpec)
}

#[derive(Serialize, Debug, Deserialize)]
pub struct MinimaxSpec {
  pub depth: u32,
  pub time_per_move: f64,
  pub evaluator: EvaluatorSpec
}

#[derive(Serialize, Debug, Deserialize)]
pub enum EvaluatorSpec {
  TerminalEvaluator,
  FeatureEvaluator(FeatureEvaluatorSpec)
}

#[derive(Serialize, Debug, Deserialize)]
pub struct FeatureEvaluatorSpec {
  pub extractor: FeatureExtractorSpec,
  pub regression: RegressionSpec
}

#[derive(Serialize, Debug, Deserialize)]
pub enum FeatureExtractorSpec {
  SubtractorFeatureExtractor(u32),
  GomokuLineFeatureExtractor
}

#[derive(Serialize, Debug, Deserialize)]
pub struct RegressionSpec {
  pub speed: f32,
  pub regularization: f32,
  pub b: Vec<f32>
}

impl AgentSpec {
  pub fn parse(s: &str) -> Option<AgentSpec> {
    serde_json::from_str(s).ok()
  }

  pub fn to_json(&self) -> String {
    serde_json::to_string_pretty(self).unwrap()
  }
}

/// First match the string with "random" or "human". If it doesn't, treat it
/// as a filename. One the file, read the contents as JSON and convert to
/// AgentSpec.
pub fn load_agent_spec(s: &str) -> Result<AgentSpec, String> {
  match s {
    "random" => Ok(AgentSpec::Random),
    "human"  => Ok(AgentSpec::Human),
    _ => {
      // Treating the string as filename.
      let mut f = File::open(s)
          .map_err(|e| format!("Error while opening file: {}", e))?;
      let mut s = String::new();
      f.read_to_string(&mut s)
          .map_err(|e| format!("Error while reading file: {}", e))?;
      AgentSpec::parse(&s).ok_or("Error while parsing AgentSpec.".to_string())
    }
  }
}

pub fn load_evaluator_spec(s: &str) -> Result<EvaluatorSpec, String> {
  let mut f = File::open(s)
      .map_err(|e| format!("Error while opening file: {}", e))?;
  let mut s = String::new();
  f.read_to_string(&mut s)
      .map_err(|e| format!("Error while reading file: {}", e))?;
  serde_json::from_str(&s).map_err(
      |e| format!("Error while parsing EvaluatorSpec: {}", e))
}

#[cfg(test)]
mod test {

use super::*;

#[test]
fn to_json_from_json() {
  let agent_spec = AgentSpec::Minimax(MinimaxSpec {
      depth: 3,
      time_per_move: 0.0,
      evaluator: EvaluatorSpec::FeatureEvaluator(FeatureEvaluatorSpec {
        extractor: FeatureExtractorSpec::SubtractorFeatureExtractor(10),
        regression: RegressionSpec {
          speed: 0.001,
          regularization: 0.001,
          b: vec![0.1, 0.2, 0.3]
        }
      })
  });

  let agent_json = agent_spec.to_json();
  println!("{}", agent_json);
  let agent_spec2 = AgentSpec::parse(&agent_json).unwrap();
  let agent_json2 = agent_spec2.to_json();

  assert_eq!(agent_json, agent_json2);
}

}  // mod test