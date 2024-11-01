use actix_web::{web, App, HttpServer, HttpResponse, Error};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use hound::{WavReader, WavWriter};
use rand::Rng;
use std::io::Write;
use std::path::PathBuf;
use std::fs::File;
use zip::{write::FileOptions, ZipWriter};

mod errors;
use errors::{AudioError, AudioResult};

fn validate_splice_params(splice_duration: f64, splice_count: i32) -> AudioResult<()> {
    if splice_duration <= 0.0 {
        return Err(AudioError::InvalidDuration("splice_duration must be positive".to_string()));
    };
    if splice_count < 1 {
        return Err(AudioError::InvalidSpliceCount("splice count must be >= 1".to_string()));
    };
    Ok(())
}

async fn process_audio(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let file_path: &str = "/tmp/received_audio.mp3";
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
                    splice_count = value.parse().unwrap_or(0);
                },
                "reverse" => {
                    let mut value = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk?;
                        value.push_str(std::str::from_utf8(&data)?);
                    }
                    reverse = value.parse().unwrap_or(false);
                },
                _ => {}
            }
        }
    }

    println!("File: {}", file_path);
    println!("Splice Duration: {}", splice_duration);
    println!("Splice Count: {}", splice_count);
    println!("Reverse: {}", reverse);
    validate_splice_params(splice_duration, splice_count);
    
    let spliced_files = process_wav(file_path, output_dir, splice_duration, splice_count)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let zip_path = "/tmp/splices.zip";
    create_zip(spliced_files, zip_path)?;

    let file_contents = std::fs::read(zip_path)?;
    Ok(HttpResponse::Ok()
        .content_type("application/zip")
        .body(file_contents))
}

fn handle_reverse() {
}

fn process_wav(input_path: &str, output_dir: &str, splice_duration: f64, splice_count: i32) -> std::io::Result<Vec<PathBuf>> {
    std::fs::create_dir_all(output_dir)?;
    let mut reader = WavReader::open(input_path).unwrap();
    let spec = reader.spec();
    let duration = reader.duration() as f64 / spec.sample_rate as f64;

    let mut rng = rand::thread_rng();
    let mut splice_files = Vec::new();

    for i in 0..splice_count {
        let start_time = rng.gen_range(0.0..duration - splice_duration);
        let start_sample = (start_time * spec.sample_rate as f64) as u32;
        let splice_samples = (splice_duration * spec.sample_rate as f64) as u32;

        let output_path = PathBuf::from(output_dir).join(format!("splice_{}.wav", i));
        let mut writer = WavWriter::create(&output_path, spec).map_err(hound_to_stdio_error)?;
        reader.seek(start_sample)?;
        for _ in 0..splice_samples {
            if let Some(sample) = reader.samples::<i16>().next() {
                let sample_value = sample.map_err(hound_to_stdio_error)?;
                writer.write_sample(sample_value).map_err(hound_to_stdio_error)?;
            } else {
                break;
            }
        }
        // not yet implemented
        handle_reverse();

        splice_files.push(output_path);
    }

    Ok(splice_files)
}

fn create_zip(files: Vec<PathBuf>, zip_path: &str) -> std::io::Result<()> {
    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    for (i, path) in files.iter().enumerate() {
        let file_name = format!("splice_{}.wav", i);
        zip.start_file(file_name, options)?;
        let contents = std::fs::read(path)?;
        zip.write_all(&contents)?;
    }

    zip.finish()?;
    Ok(())
}

fn hound_to_stdio_error(e: hound::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/process", web::post().to(process_audio))
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}