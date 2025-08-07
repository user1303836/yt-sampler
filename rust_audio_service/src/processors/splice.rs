use std::path::PathBuf;
use std::time::Instant;
use hound::{WavReader, WavWriter};
use rand::Rng;
use log::info;

use crate::errors::{AudioError, AudioResult};
use super::{AudioProcessor, ProcessorConfig, ProcessingResult, ProcessingMetadata};

pub struct SpliceProcessor;

impl SpliceProcessor {
    pub fn new() -> Self {
        Self
    }

    fn reverse_samples(samples: &mut [i16]) {
        samples.reverse();
    }

    fn validate_splice_params(splice_duration: f64, splice_count: i32) -> AudioResult<()> {
        if splice_duration <= 0.0 {
            return Err(AudioError::InvalidDuration("splice_duration must be positive".to_string()));
        }
        if splice_count < 1 {
            return Err(AudioError::InvalidSpliceCount("splice count must be >= 1".to_string()));
        }
        Ok(())
    }
}

impl AudioProcessor for SpliceProcessor {
    fn process(&self, input_path: &str, output_dir: &str, config: &ProcessorConfig) -> AudioResult<ProcessingResult> {
        let start_time = Instant::now();
        
        let (duration, count, reverse) = match config {
            ProcessorConfig::Splice { duration, count, reverse } => (*duration, *count, *reverse),
        };

        self.validate_config(config)?;
        
        std::fs::create_dir_all(output_dir).map_err(|e| AudioError::IoError(e))?;
        let mut reader = WavReader::open(input_path).map_err(|e| AudioError::WavError(e))?;
        let spec = reader.spec();
        let total_duration = reader.duration() as f64 / spec.sample_rate as f64;

        info!("Processing splice - Duration: {}, Count: {}, Reverse: {}", duration, count, reverse);

        let mut rng = rand::thread_rng();
        let mut splice_files = Vec::new();

        for i in 0..count {
            let start_time_splice = rng.gen_range(0.0..total_duration - duration);
            let start_sample = (start_time_splice * spec.sample_rate as f64) as u32;
            let splice_samples = (duration * spec.sample_rate as f64) as u32;

            let output_path = PathBuf::from(output_dir).join(format!("splice_{}.wav", i));
            let mut writer = WavWriter::create(&output_path, spec).map_err(|e| AudioError::WavError(e))?;
            reader.seek(start_sample).map_err(|e| AudioError::IoError(e))?;
            
            let mut samples = Vec::with_capacity(splice_samples as usize);
            for _ in 0..splice_samples {
                if let Some(sample) = reader.samples::<i16>().next() {
                    let sample_value = sample.map_err(|e| AudioError::WavError(e))?;
                    samples.push(sample_value);
                } else {
                    break;
                }
            }
            
            if reverse {
                Self::reverse_samples(&mut samples);
            }
            
            for sample in samples {
                writer.write_sample(sample).map_err(|e| AudioError::WavError(e))?;
            }

            splice_files.push(output_path);
        }

        let processing_time = start_time.elapsed();
        
        Ok(ProcessingResult {
            files: splice_files,
            metadata: ProcessingMetadata {
                processor_type: self.processor_type().to_string(),
                input_duration: total_duration,
                sample_rate: spec.sample_rate,
                channels: spec.channels,
                processing_time_ms: processing_time.as_millis() as u64,
            },
        })
    }

    fn validate_config(&self, config: &ProcessorConfig) -> AudioResult<()> {
        match config {
            ProcessorConfig::Splice { duration, count, .. } => {
                Self::validate_splice_params(*duration, *count)
            }
        }
    }

    fn processor_type(&self) -> &'static str {
        "splice"
    }
}