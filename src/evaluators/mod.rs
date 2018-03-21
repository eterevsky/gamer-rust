mod annealing;
pub mod features;
mod regression;
mod reinforce;
mod sampler;
mod terminal;
pub mod train_subtractor_eval;

pub use self::annealing::AnnealingTrainer;
pub use self::features::FeatureEvaluator;
pub use self::regression::LinearRegressionTanh;
pub use self::reinforce::ReinforceTrainer;
pub use self::sampler::SamplerEvaluator;
pub use self::terminal::TerminalEvaluator;