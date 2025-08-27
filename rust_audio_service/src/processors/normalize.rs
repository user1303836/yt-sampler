use std::path::PathBuf;
use std::time::Instant;
use hound::{WavReader, WavWriter};
use rand::Rng;
use log::info;

use crate::errors::{AudioError, AudioResult};
use super::{AudioProcessor, ProcessorConfig, ProcessingResult, ProcessingMetadata};

pub struct NormalizeProcessor;

impl NormalizeProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Find the peak (maximum absolute value) in a set of samples
    fn find_peak(samples: &[i16]) -> f64 {
        samples
            .iter()
            .map(|&sample| (sample as f64).abs())
            .fold(0.0, f64::max)
            / i16::MAX as f64  // Convert to 0.0-1.0 range
    }

    /// Apply normalization gain to samples
    fn apply_gain(samples: &mut [i16], gain: f64) {
        for sample in samples {
            let normalized = (*sample as f64 * gain).round();
            *sample = normalized.clamp(i16::MIN as f64, i16::MAX as f64) as i16;
        }
    }

    /// Normalize a single file
    fn normalize_file(input_path: &str, output_path: &PathBuf, target_level: f64) -> AudioResult<()> {
        let mut reader = WavReader::open(input_path).map_err(|e| AudioError::WavError(e))?;
        let spec = reader.spec();
        
        // Read all samples into memory
        let samples: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
        let mut samples = samples.map_err(|e| AudioError::WavError(e))?;
        
        if samples.is_empty() {
            return Err(AudioError::ProcessingError("No audio data found".to_string()));
        }

        // Find peak level
        let peak_level = Self::find_peak(&samples);
        
        if peak_level == 0.0 {
            return Err(AudioError::ProcessingError("Audio is silent (no signal detected)".to_string()));
        }

        // Calculate normalization gain
        let gain = target_level / peak_level;
        
        info!("Normalizing: peak={:.3}, target={:.3}, gain={:.3}x", 
              peak_level, target_level, gain);

        // Apply gain to all samples
        Self::apply_gain(&mut samples, gain);

        // Write normalized audio
        let mut writer = WavWriter::create(output_path, spec).map_err(|e| AudioError::WavError(e))?;
        for sample in samples {
            writer.write_sample(sample).map_err(|e| AudioError::WavError(e))?;
        }

        Ok(())
    }

    /// Create normalized splices (like the splice processor, but with normalization)
    fn create_normalized_splices(
        input_path: &str, 
        output_dir: &str, 
        target_level: f64,
        splice_duration: f64,
        splice_count: i32,
        apply_to_splices: bool
    ) -> AudioResult<Vec<PathBuf>> {
        std::fs::create_dir_all(output_dir).map_err(|e| AudioError::IoError(e))?;
        let mut reader = WavReader::open(input_path).map_err(|e| AudioError::WavError(e))?;
        let spec = reader.spec();
        let total_duration = reader.duration() as f64 / spec.sample_rate as f64;

        let mut rng = rand::thread_rng();
        let mut splice_files = Vec::new();

        for i in 0..splice_count {
            let start_time = rng.gen_range(0.0..total_duration - splice_duration);
            let start_sample = (start_time * spec.sample_rate as f64) as u32;
            let splice_samples_count = (splice_duration * spec.sample_rate as f64) as u32;

            let output_path = PathBuf::from(output_dir).join(format!("normalized_splice_{}.wav", i));
            let mut writer = WavWriter::create(&output_path, spec).map_err(|e| AudioError::WavError(e))?;
            
            reader.seek(start_sample).map_err(|e| AudioError::IoError(e))?;
            
            let mut samples = Vec::with_capacity(splice_samples_count as usize);
            for _ in 0..splice_samples_count {
                if let Some(sample) = reader.samples::<i16>().next() {
                    let sample_value = sample.map_err(|e| AudioError::WavError(e))?;
                    samples.push(sample_value);
                } else {
                    break;
                }
            }
            
            if samples.is_empty() {
                continue;
            }

            // Apply normalization if requested
            if apply_to_splices {
                let peak_level = Self::find_peak(&samples);
                if peak_level > 0.0 {
                    let gain = target_level / peak_level;
                    Self::apply_gain(&mut samples, gain);
                }
            }
            
            for sample in samples {
                writer.write_sample(sample).map_err(|e| AudioError::WavError(e))?;
            }

            splice_files.push(output_path);
        }

        Ok(splice_files)
    }
}

impl AudioProcessor for NormalizeProcessor {
    fn process(&self, input_path: &str, output_dir: &str, config: &ProcessorConfig) -> AudioResult<ProcessingResult> {
        let start_time = Instant::now();
        
        let (target_level, apply_to_splices) = match config {
            ProcessorConfig::Normalize { target_level, apply_to_splices } => (*target_level, *apply_to_splices),
            _ => return Err(AudioError::ProcessingError("Invalid config for NormalizeProcessor".to_string())),
        };

        self.validate_config(config)?;
        
        std::fs::create_dir_all(output_dir).map_err(|e| AudioError::IoError(e))?;
        
        // Read input file metadata
        let reader = WavReader::open(input_path).map_err(|e| AudioError::WavError(e))?;
        let spec = reader.spec();
        let total_duration = reader.duration() as f64 / spec.sample_rate as f64;
        drop(reader);

        info!("Processing normalize - Target level: {}, Apply to splices: {}", target_level, apply_to_splices);

        let output_files = if apply_to_splices {
            // Create normalized splices (this is a hybrid mode - creates splices AND normalizes them)
            // For simplicity, we'll create 5 splices of 2 seconds each
            // In a real implementation, you might want splice parameters in the config
            Self::create_normalized_splices(
                input_path, 
                output_dir, 
                target_level,
                2.0,  // 2 second splices
                5,    // 5 splices
                true
            )?
        } else {
            // Just normalize the entire file
            let output_path = PathBuf::from(output_dir).join("normalized_audio.wav");
            Self::normalize_file(input_path, &output_path, target_level)?;
            vec![output_path]
        };

        let processing_time = start_time.elapsed();
        
        Ok(ProcessingResult {
            files: output_files,
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
            ProcessorConfig::Normalize { target_level, .. } => {
                if *target_level <= 0.0 || *target_level > 1.0 {
                    return Err(AudioError::ProcessingError(
                        "target_level must be between 0.0 and 1.0 (where 1.0 = maximum level)".to_string()
                    ));
                }
                Ok(())
            },
            _ => Err(AudioError::ProcessingError("Invalid config for NormalizeProcessor".to_string())),
        }
    }

    fn processor_type(&self) -> &'static str {
        "normalize"
    }
}