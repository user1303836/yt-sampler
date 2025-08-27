use actix_web::{web, App, HttpServer, HttpResponse, Error};
use actix_multipart::Multipart;
use actix_files as fs;
use futures::{StreamExt, TryStreamExt};
use std::io::Write;
use log::{info, error};

mod errors;
mod processors;
mod api;
mod utils;

use errors::AudioError;
use processors::{ProcessorConfig, splice::SpliceProcessor, AudioProcessor};
use utils::{create_zip_from_result, cleanup_temp_files};

// Legacy endpoint for backward compatibility with Go CLI
async fn process_audio(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let file_path: &str = "/tmp/received_audio.wav";
    let output_dir: &str = "/tmp/splices";
    let mut splice_duration: f64 = 0.0;
    let mut splice_count: i32 = 0;
    let mut reverse: bool = false;

    // Look at multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();

        if let Some(name) = content_disposition.expect("Something bad happened...").get_name() {
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

    info!("Legacy endpoint - Processing audio - File: {}, Duration: {}, Count: {}, Reverse: {}", 
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
                    cleanup_temp_files(file_path, &result.files, zip_path);
                    
                    Ok(HttpResponse::Ok()
                        .content_type("application/zip")
                        .body(file_contents))
                },
                Err(e) => {
                    error!("Failed to create ZIP: {}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
            }
        },
        Err(e) => {
            error!("Audio processing failed: {}", e);
            Ok(HttpResponse::BadRequest().finish())
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    api::v1::init_start_time();
    
    info!("Starting audio service on 127.0.0.1:8081");
    info!("Web interface: http://127.0.0.1:8081");
    info!("Legacy endpoint: POST /process");
    info!("New API endpoints: /api/v1/health, /api/v1/audio/splice/multipart, /api/v1/audio/normalize/multipart");
    
    HttpServer::new(|| {
        App::new()
            .route("/process", web::post().to(process_audio))  // Legacy endpoint
            .configure(api::v1::config)  // New v1 API endpoints
            .service(fs::Files::new("/", "./web").index_file("index.html"))  // Static web files
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}