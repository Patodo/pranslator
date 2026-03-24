use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use reqwest::Client;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;

use crate::dictionary::{DICT_FILENAME, DICT_FILE_SIZE, DICT_URL, DICT_ZIP_FILENAME};

/// Download and extract the MDX dictionary.
/// Reports progress via Tauri events.
pub async fn download_dictionary(app: AppHandle, config_dir: &Path) -> Result<()> {
    let client = Client::new();

    // Create temp directory for download
    let temp_dir = std::env::temp_dir().join("pranslator-dict-download");
    fs::create_dir_all(&temp_dir)
        .with_context(|| "Failed to create temp directory")?;

    let zip_path = temp_dir.join(DICT_ZIP_FILENAME);

    // Download the zip file with progress
    let response = client
        .get(DICT_URL)
        .send()
        .await
        .with_context(|| "Failed to start download")?;

    if !response.status().is_success() {
        return Err(anyhow!("Download failed with status: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(DICT_FILE_SIZE);
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut file = File::create(&zip_path)
        .with_context(|| "Failed to create temp file")?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.with_context(|| "Failed to read chunk")?;
        file.write_all(&chunk)
            .with_context(|| "Failed to write chunk")?;
        downloaded += chunk.len() as u64;

        // Calculate and emit progress (0-100)
        let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
        let _ = app.emit("dictionary-download-progress", progress);
    }

    // Extract the MDX file from zip
    extract_mdx_from_zip(&zip_path, config_dir)?;

    // Cleanup temp files
    let _ = fs::remove_file(&zip_path);
    let _ = fs::remove_dir(&temp_dir);

    Ok(())
}

/// Extract the MDX file from the downloaded zip.
fn extract_mdx_from_zip(zip_path: &Path, config_dir: &Path) -> Result<()> {
    let file = File::open(zip_path)
        .with_context(|| "Failed to open zip file")?;

    let mut archive = ZipArchive::new(file)
        .with_context(|| "Failed to read zip archive")?;

    // Find and extract the .mdx file
    let mut found = false;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .with_context(|| format!("Failed to access file {} in archive", i))?;

        let name = file.name();
        if name.ends_with(".mdx") {
            let dest_path = config_dir.join(DICT_FILENAME);

            // Ensure config directory exists
            fs::create_dir_all(config_dir)
                .with_context(|| "Failed to create config directory")?;

            let mut dest_file = File::create(&dest_path)
                .with_context(|| "Failed to create MDX file")?;

            io::copy(&mut file, &mut dest_file)
                .with_context(|| "Failed to extract MDX file")?;

            found = true;
            break;
        }
    }

    if !found {
        return Err(anyhow!("No .mdx file found in the archive"));
    }

    Ok(())
}

/// Delete the dictionary file.
pub fn delete_dictionary(config_dir: &Path) -> Result<()> {
    let dict_path = config_dir.join(DICT_FILENAME);

    if dict_path.exists() {
        fs::remove_file(&dict_path)
            .with_context(|| "Failed to delete dictionary file")?;
    }

    Ok(())
}
