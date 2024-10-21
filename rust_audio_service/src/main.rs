use actix_web::{web, App, HttpServer, HttpResponse, Error};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use hound::{WavReader, WavWriter, WavSpec};
use rand::Rng;
use std::io::Write;
use std::path::PathBuf;
use std::fs::File;
use zip::{write::FileOptions, ZipWriter};

async fn process_audio(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let file_path: &str = "/tmp/received_audio.mp3";
    let output_dir: &str = "/tmp/splices";
    let mut splice_duration: f64 = 0.0;
    let mut splice_count: i32 = 0;

    // Look at multipart stream and do stuff
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();

        if let Some(name) = content_disposition.expect("Something bad happened...").get_name() {
            match name {
                "file" => {
                    let mut f = std::fs::File::create(file_path)?;
                    while let Some(chunk) = field.next().await {
                        let data = chunk?;
                        f.write_all(&data)?;
                    }
                },
                "spliceDuration" => {
                    let mut value = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk?;
                        value.push_str(std::str::from_utf8(&data)?);
                    }
                    splice_duration = value.parse().unwrap_or(0.0);
                },
                "spliceCount" => {
                    let mut value = String::new();
                    while let Some(chunk) = field.next().await {
                        let data = chunk?;
                        value.push_str(std::str::from_utf8(&data)?);
                    }
                    splice_count = value.parse().unwrap_or(0);
                },
                _ => {}
            }
        }
    }

    println!("File: {}", file_path);
    println!("Splice Duration: {}", splice_duration);
    println!("Splice Count: {}", splice_count);

    // Return file
    let file_contents = std::fs::read(file_path)?;
    Ok(HttpResponse::Ok()
        .content_type("audio/mpeg")
        .body(file_contents))
}

fn process_wav(input_path: &str, output_dir: &str, splice_duration: f64, splice_count: i32) -> std::io::Result<Vec<PathBuf>> {
    std::fs::create_dir_all(output_dir)?;
    let reader = WavReader::open(input_path).unwrap();
    let spec = reader.spec();
    let duration = reader.duration() as f64 / spec.sample_rate as f64;

    let mut rng = rand::thread_rng();
    let splice_files = Vec::new();

    for _i in 0..splice_count {
        let start = rng.gen_range(0.0..duration - splice_duration);
        let start_sample = (start * spec.sample_rate as f64) as u32;
        let splice_samples = (splice_duration * spec.sample_rate as f64) as u32; 
    }

    Ok(splice_files)
}

fn create_zip<T: zip::write::FileOptionExtension + std::marker::Copy>(files: Vec<PathBuf>, zip_path: &str) -> std::io::Result<()> {
    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<'_, T> = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for (i, path) in files.iter().enumerate() {
        let file_name = format!("splice_{}.wav", i);
        zip.start_file(file_name, options)?;
        let contents = std::fs::read(path)?;
        zip.write_all(&contents)?;
    }

    zip.finish()?;
    Ok(())
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