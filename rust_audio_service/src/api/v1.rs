use actix_web::{web, HttpResponse, Result, Error};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use std::io::Write;
use log::{info, error};
use std::time::SystemTime;

use crate::processors::{ProcessorConfig, splice::SpliceProcessor, AudioProcessor};
use crate::api::{ProcessAudioRequest, ProcessAudioResponse, HealthResponse, ErrorResponse};
use crate::errors::AudioError;
use crate::utils::create_zip_from_result;

static START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health_check))
            .route("/audio/splice", web::post().to(process_audio_json))
            .route("/audio/splice/multipart", web::post().to(process_audio_multipart))
    );
}

pub fn init_start_time() {
    START_TIME.set(SystemTime::now()).ok();
}

async fn health_check() -> Result<HttpResponse> {
    let now = SystemTime::now();
    let start_time = START_TIME.get().unwrap_or(&now);
    let uptime = SystemTime::now()
        .duration_since(*start_time)
        .unwrap_or_default()
        .as_secs();

    Ok(HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
    }))
}

async fn process_audio_json(req: web::Json<ProcessAudioRequest>) -> Result<HttpResponse> {
    info!("Processing JSON audio request: {:?}", req.config);
    
    // For JSON requests, we need to handle file upload differently
    // This is a simplified version - in a real implementation you'd want
    // to use a different approach for file uploads with JSON
    let error_response = ErrorResponse {
        error: "JSON file upload not yet implemented. Use /audio/splice/multipart endpoint.".to_string(),
        error_type: "NotImplemented".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    Ok(HttpResponse::NotImplemented().json(error_response))
}

async fn process_audio_multipart(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let file_path = "/tmp/received_audio.wav";
    let output_dir = "/tmp/splices";
    let mut splice_duration: f64 = 0.0;
    let mut splice_count: i32 = 0;
    let mut reverse: bool = false;

    // Parse multipart data
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();

        if let Some(name) = content_disposition.expect("Invalid content disposition").get_name() {
            match name {
                "file" => {
                    let mut f = std::fs::File::create(file_path)?;
                    while let Some(chunk) = field.next().await {
                        let data = chunk.map_err(|e| AudioError::ProcessingError(e.to_string()))?;
                        f.write_all(&data)?;
                    }
                },
                "spliceDuration" => {
                    let mut value = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk.map_err(|e| AudioError::InvalidDuration(e.to_string()))?;
                        value.push_str(std::str::from_utf8(&data).map_err(|e| AudioError::ProcessingError(e.to_string()))?);
                    }
                    splice_duration = value.parse().map_err(|_| AudioError::InvalidDuration("Invalid splice duration format".to_string()))?;
                },
                "spliceCount" => {
                    let mut value = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk?;
                        value.push_str(std::str::from_utf8(&data)?);
                    }
                    splice_count = value.parse().map_err(|_| AudioError::InvalidSpliceCount("Invalid splice count format".to_string()))?;
                },
                "reverse" => {
                    let mut value = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk?;
                        value.push_str(std::str::from_utf8(&data)?);
                    }
                    reverse = value.parse().map_err(|_| AudioError::ProcessingError("Invalid reverse flag format".to_string()))?;
                },
                _ => {}
            }
        }
    }

    info!("Processing audio - File: {}, Duration: {}, Count: {}, Reverse: {}", 
          file_path, splice_duration, splice_count, reverse);

    // Create config and process using the new architecture
    let config = ProcessorConfig::Splice {
        duration: splice_duration,
        count: splice_count,
        reverse,
    };

    let processor = SpliceProcessor::new();
    
    match processor.process(file_path, output_dir, &config) {
        Ok(result) => {
            let zip_path = "/tmp/splices.zip";
            
            match create_zip_from_result(&result, zip_path) {
                Ok(_) => {
                    let file_contents = std::fs::read(zip_path)?;
                    
                    // Cleanup temporary files
                    crate::utils::cleanup_temp_files(file_path, &result.files, zip_path);
                    
                    Ok(HttpResponse::Ok()
                        .content_type("application/zip")
                        .body(file_contents))
                },
                Err(e) => {
                    error!("Failed to create ZIP: {}", e);
                    Ok(HttpResponse::InternalServerError().json(ProcessAudioResponse::error(e.to_string())))
                }
            }
        },
        Err(e) => {
            error!("Audio processing failed: {}", e);
            Ok(HttpResponse::BadRequest().json(ProcessAudioResponse::error(e.to_string())))
        }
    }
}