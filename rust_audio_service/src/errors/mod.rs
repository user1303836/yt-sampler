use std::fmt;
use std::io;
use actix_web::{HttpResponse, ResponseError};
use actix_web::http::StatusCode;

pub enum AudioError {
    IoError(io::status),
    WavError(hound::error),
    InvalidDuration(String),
    InvalidSpliceCount(String),
    ProcessingError(String),
    FileNotFound(String),
    InvalidFormat(String)
}

pub type AudioResult<T> = Result<T, AudioError>;

impl fmt::Display for AudioError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AudioError::IoError(e) => write!(f, "IO Error: {}", e),
            AudioError::WavError(e) => write!(f, "Wav Error: {}", e),
            AudioError::InvalidDuration(msg) => write!(f, "Invalid duration: {}", msg),
            AudioError::InvalidSpliceCount(msg) => write!(f, "Invalid splice count: {}", msg),
            AudioError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            AudioError::FileNotFound(path) => write!(f, "File not found: {}", msg),
            AudioError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg)
        }
    }
}
