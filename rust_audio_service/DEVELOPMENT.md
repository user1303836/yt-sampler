# Developer Guide - Rust Audio Service

## Architecture Overview

The Rust Audio Service is built using a modular, trait-based architecture that separates concerns and makes it easy to add new audio processing capabilities.

### Core Components

```
src/
├── main.rs              # HTTP server and legacy endpoint
├── errors/
│   └── mod.rs          # Error types and handling
├── processors/
│   ├── mod.rs          # AudioProcessor trait and types
│   └── splice.rs       # SpliceProcessor implementation
├── api/
│   ├── mod.rs          # API request/response types
│   └── v1.rs           # Version 1 API endpoints
└── utils.rs            # Utility functions
```

---

## AudioProcessor Trait System

### Core Trait

```rust
pub trait AudioProcessor {
    fn process(&self, input_path: &str, output_dir: &str, config: &ProcessorConfig) -> AudioResult<ProcessingResult>;
    fn validate_config(&self, config: &ProcessorConfig) -> AudioResult<()>;
    fn processor_type(&self) -> &'static str;
}
```

### Configuration System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProcessorConfig {
    Splice {
        duration: f64,
        count: i32,
        reverse: bool,
    },
    // Add new processor configs here
}
```

### Processing Result

```rust
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
```

---

## Adding New Audio Effects

### Step 1: Define Configuration

Add your processor configuration to the `ProcessorConfig` enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProcessorConfig {
    Splice { /* ... */ },
    Reverb {
        wet_level: f64,
        dry_level: f64,
        decay_time: f64,
    },
}
```

### Step 2: Create Processor

Create a new file `src/processors/reverb.rs`:

```rust
use std::path::PathBuf;
use std::time::Instant;
use crate::errors::{AudioError, AudioResult};
use super::{AudioProcessor, ProcessorConfig, ProcessingResult, ProcessingMetadata};

pub struct ReverbProcessor;

impl ReverbProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl AudioProcessor for ReverbProcessor {
    fn process(&self, input_path: &str, output_dir: &str, config: &ProcessorConfig) -> AudioResult<ProcessingResult> {
        let start_time = Instant::now();
        
        let (wet_level, dry_level, decay_time) = match config {
            ProcessorConfig::Reverb { wet_level, dry_level, decay_time } => (*wet_level, *dry_level, *decay_time),
            _ => return Err(AudioError::ProcessingError("Invalid config for ReverbProcessor".to_string())),
        };

        self.validate_config(config)?;
        
        // Implement reverb processing logic here
        // ...
        
        let processing_time = start_time.elapsed();
        
        Ok(ProcessingResult {
            files: vec![/* output files */],
            metadata: ProcessingMetadata {
                processor_type: self.processor_type().to_string(),
                input_duration: 0.0, // Calculate from input
                sample_rate: 44100,  // Get from input
                channels: 2,         // Get from input
                processing_time_ms: processing_time.as_millis() as u64,
            },
        })
    }

    fn validate_config(&self, config: &ProcessorConfig) -> AudioResult<()> {
        match config {
            ProcessorConfig::Reverb { wet_level, dry_level, .. } => {
                if *wet_level < 0.0 || *wet_level > 1.0 {
                    return Err(AudioError::ProcessingError("wet_level must be between 0.0 and 1.0".to_string()));
                }
                if *dry_level < 0.0 || *dry_level > 1.0 {
                    return Err(AudioError::ProcessingError("dry_level must be between 0.0 and 1.0".to_string()));
                }
                Ok(())
            },
            _ => Err(AudioError::ProcessingError("Invalid config for ReverbProcessor".to_string())),
        }
    }

    fn processor_type(&self) -> &'static str {
        "reverb"
    }
}
```

### Step 3: Register Processor

Add the module to `src/processors/mod.rs`:

```rust
pub mod splice;
pub mod reverb;  // Add this line

// Export the processor
pub use reverb::ReverbProcessor;
```

### Step 4: Add API Endpoint

Add a new endpoint in `src/api/v1.rs`:

```rust
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health_check))
            .route("/audio/splice/multipart", web::post().to(process_audio_multipart))
            .route("/audio/reverb/multipart", web::post().to(process_reverb_multipart))  // Add this
    );
}

async fn process_reverb_multipart(/* ... */) -> Result<HttpResponse, Error> {
    // Similar to splice implementation
    let processor = ReverbProcessor::new();
    // ... handle multipart data and process
}
```

---

## Audio Processing Utilities

### Working with WAV Files

```rust
use hound::{WavReader, WavWriter};

// Reading WAV files
let mut reader = WavReader::open(input_path)?;
let spec = reader.spec();
let duration = reader.duration() as f64 / spec.sample_rate as f64;

// Writing WAV files
let mut writer = WavWriter::create(output_path, spec)?;
for sample in samples {
    writer.write_sample(sample)?;
}
```

### Sample Manipulation

```rust
// Read samples into a buffer
let mut samples: Vec<i16> = Vec::new();
for sample_result in reader.samples::<i16>() {
    samples.push(sample_result?);
}

// Apply effects
fn apply_gain(samples: &mut [i16], gain: f64) {
    for sample in samples {
        *sample = (*sample as f64 * gain) as i16;
    }
}

fn reverse_samples(samples: &mut [i16]) {
    samples.reverse();
}
```

### Utility Functions

Available utility functions in `src/utils.rs`:

```rust
// Create ZIP from processing result
create_zip_from_result(&result, zip_path)?;

// Clean up temporary files
cleanup_temp_files(input_file, &result.files, zip_file);
```

---

## Error Handling

### Custom Error Types

```rust
#[derive(Debug)]
pub enum AudioError {
    IoError(io::Error),
    WavError(hound::Error),
    InvalidDuration(String),
    InvalidSpliceCount(String),
    ProcessingError(String),
    FileNotFound(String),
    InvalidFormat(String)
}
```

### Error Propagation

Use `?` operator for clean error propagation:

```rust
fn process_audio() -> AudioResult<ProcessingResult> {
    let reader = WavReader::open(path)?;  // Automatically converts hound::Error
    let spec = reader.spec();
    // ...
    Ok(result)
}
```

---

## Testing Guidelines

### Unit Testing Processors

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::ProcessorConfig;

    #[test]
    fn test_splice_processor_validation() {
        let processor = SpliceProcessor::new();
        
        let valid_config = ProcessorConfig::Splice {
            duration: 2.0,
            count: 5,
            reverse: false,
        };
        
        assert!(processor.validate_config(&valid_config).is_ok());
        
        let invalid_config = ProcessorConfig::Splice {
            duration: -1.0,  // Invalid
            count: 5,
            reverse: false,
        };
        
        assert!(processor.validate_config(&invalid_config).is_err());
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_splice_endpoint() {
    // Test the actual HTTP endpoint
    // Use actix-web test utilities
}
```

---

## Performance Considerations

### Memory Usage

- Process audio in chunks for large files
- Use `Vec::with_capacity()` when size is known
- Clean up temporary files promptly

### CPU Usage

- Consider using `rayon` for parallel processing
- Profile audio processing algorithms
- Use efficient audio libraries

### I/O Optimization

- Batch file operations
- Use appropriate buffer sizes
- Consider streaming for large files

---

## Future Extensions

### Effect Chaining

```rust
pub struct EffectChain {
    processors: Vec<Box<dyn AudioProcessor>>,
}

impl EffectChain {
    pub fn add_processor(&mut self, processor: Box<dyn AudioProcessor>) {
        self.processors.push(processor);
    }
    
    pub fn process(&self, input: AudioBuffer) -> AudioResult<AudioBuffer> {
        let mut buffer = input;
        for processor in &self.processors {
            buffer = processor.process_buffer(buffer)?;
        }
        Ok(buffer)
    }
}
```

### Async Processing

```rust
pub async fn process_async(&self, config: ProcessorConfig) -> AudioResult<ProcessingResult> {
    // Long-running processing with progress updates
    // Use tokio channels for progress reporting
}
```

### Plugin System

```rust
pub trait AudioPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn create_processor(&self) -> Box<dyn AudioProcessor>;
}
```