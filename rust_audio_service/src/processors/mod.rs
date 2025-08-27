use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::errors::AudioResult;

pub mod splice;
pub mod normalize;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProcessorConfig {
    Splice {
        duration: f64,
        count: i32,
        reverse: bool,
    },
    Normalize {
        target_level: f64,  // Target peak level (0.0 to 1.0, where 1.0 = 0dB)
        apply_to_splices: bool,  // If true, normalize each splice individually
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub files: Vec<PathBuf>,
    pub metadata: ProcessingMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    pub processor_type: String,
    pub input_duration: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub processing_time_ms: u64,
}

pub trait AudioProcessor {
    fn process(&self, input_path: &str, output_dir: &str, config: &ProcessorConfig) -> AudioResult<ProcessingResult>;
    fn validate_config(&self, config: &ProcessorConfig) -> AudioResult<()>;
    fn processor_type(&self) -> &'static str;
}