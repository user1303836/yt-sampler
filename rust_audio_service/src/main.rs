use actix_web::{web, App, HttpServer, HttpResponse, Error};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use std::io::Write;

async fn process_audio(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // Temporary file path
    let file_path = "/tmp/received_audio.mp3";

    // Iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut f = std::fs::File::create(file_path)?;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            f.write_all(&data)?;
        }
    }

    println!("Successfully received and read the file: {}", file_path);

    // Send the file back
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