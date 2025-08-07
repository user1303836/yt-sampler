use serde::{Deserialize, Serialize};
use crate::processors::{ProcessorConfig, ProcessingResult};

pub mod v1;

#[derive(Debug, Deserialize)]
pub struct ProcessAudioRequest {
    pub config: ProcessorConfig,
}

#[derive(Debug, Serialize)]
pub struct ProcessAudioResponse {
    pub success: bool,
    pub result: Option<ProcessingResult>,
    pub error: Option<String>,
}

impl ProcessAudioResponse {
    pub fn success(result: ProcessingResult) -> Self {
        Self {
            success: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            result: None,
            error: Some(message),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub error_type: String,
    pub timestamp: String,
}