use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use zip::{write::FileOptions, ZipWriter};
use log::warn;

use crate::processors::ProcessingResult;

pub fn create_zip_from_result(result: &ProcessingResult, zip_path: &str) -> std::io::Result<()> {
    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    for (i, path) in result.files.iter().enumerate() {
        // Use the actual filename from the path, or create a generic name if needed
        let file_name = if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            name.to_string()
        } else {
            format!("output_{}.wav", i)
        };
        zip.start_file(file_name, options)?;
        let contents = std::fs::read(path)?;
        zip.write_all(&contents)?;
    }

    zip.finish()?;
    Ok(())
}

pub fn cleanup_temp_files(input_file: &str, splice_files: &[PathBuf], _zip_file: &str) {
    // Remove input file
    if let Err(e) = std::fs::remove_file(input_file) {
        warn!("Failed to remove input file {}: {}", input_file, e);
    }
    
    // Remove splice files
    for file in splice_files {
        if let Err(e) = std::fs::remove_file(file) {
            warn!("Failed to remove splice file {:?}: {}", file, e);
        }
    }
    
    // Remove output directory if empty
    if let Some(parent) = splice_files.first().and_then(|p| p.parent()) {
        if let Err(e) = std::fs::remove_dir(parent) {
            warn!("Failed to remove output directory {:?}: {}", parent, e);
        }
    }
    
    log::info!("Cleanup completed for processing session");
}