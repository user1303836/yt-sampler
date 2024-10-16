use actix_web::{web, App, HttpServer, HttpResponse, Error};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use std::io::Write;

async fn process_audio(mut payload: Multipart) -> Result<HttpResponse, Error> {
    let file_path = "/tmp/received_audio.mp3";
    let mut splice_duration = 0.0;
    let mut splice_count = 0;

    // Look at multipart stream and do stuff
    while let Ok(Some(mut field)) = payload.try_next().await {
        // let mut f = std::fs::File::create(file_path)?;
        // while let Some(chunk) = field.next().await {
        //     let data = chunk?;
        //     f.write_all(&data)?;
        // }

        let content_disposition = field.content_disposition();

        if let Some(name) = content_disposition.expect("reason").get_name() {
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

    println!("Successfully received and read file: {}", file_path);
    println!("Splice Duration: {}", splice_duration);
    println!("Splice Count: {}", splice_count);

    // Return file
    let file_contents = std::fs::read(file_path)?;
    Ok(HttpResponse::Ok()
        .content_type("audio/mpeg")
        .body(file_contents))
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