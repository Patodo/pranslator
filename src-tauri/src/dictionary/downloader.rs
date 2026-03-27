use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use reqwest::Client;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;

use crate::dictionary::{DICT_FILENAME, DICT_FILE_SIZE, DICT_URL, DICT_ZIP_FILENAME};

/// Global flag to cancel download
static CANCEL_DOWNLOAD: AtomicBool = AtomicBool::new(false);

/// Cancel the current download
pub fn cancel_download() {
    CANCEL_DOWNLOAD.store(true, Ordering::SeqCst);
}

/// Reset the cancel flag
fn reset_cancel_flag() {
    CANCEL_DOWNLOAD.store(false, Ordering::SeqCst);
}

/// Check if download was cancelled
fn is_cancelled() -> bool {
    CANCEL_DOWNLOAD.load(Ordering::SeqCst)
}

/// Progress event payload sent to frontend
#[derive(serde::Serialize, Clone)]
pub struct DownloadProgress {
    pub progress: u8,
    pub downloaded: u64,
    pub total: u64,
    pub speed: String,
}

/// Download and extract the MDX dictionary.
/// Reports progress via Tauri events.
pub async fn download_dictionary(app: AppHandle, config_dir: &Path) -> Result<()> {
    reset_cancel_flag();

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

    let start_time = Instant::now();
    let mut last_emit_time = Instant::now();

    while let Some(chunk) = stream.next().await {
        // Check for cancellation
        if is_cancelled() {
            // Cleanup temp files
            let _ = fs::remove_file(&zip_path);
            let _ = fs::remove_dir(&temp_dir);
            return Err(anyhow!("Download cancelled by user"));
        }

        let chunk = chunk.with_context(|| "Failed to read chunk")?;
        file.write_all(&chunk)
            .with_context(|| "Failed to write chunk")?;
        downloaded += chunk.len() as u64;

        // Emit progress event (throttled to ~10Hz)
        let now = Instant::now();
        if now.duration_since(last_emit_time) >= Duration::from_millis(100) {
            let progress = ((downloaded as f64 / total_size as f64) * 100.0).min(100.0) as u8;

            // Calculate speed
            let elapsed = now.duration_since(start_time).as_secs_f64();
            let speed = if elapsed > 0.0 {
                let bytes_per_sec = downloaded as f64 / elapsed;
                format_speed(bytes_per_sec)
            } else {
                "0 B/s".to_string()
            };

            let payload = DownloadProgress {
                progress,
                downloaded,
                total: total_size,
                speed,
            };

            let _ = app.emit("dictionary-download-progress", payload);
            last_emit_time = now;
        }
    }

    // Check again after download completes
    if is_cancelled() {
        let _ = fs::remove_file(&zip_path);
        let _ = fs::remove_dir(&temp_dir);
        return Err(anyhow!("Download cancelled by user"));
    }

    // Emit final progress
    let _ = app.emit("dictionary-download-progress", DownloadProgress {
        progress: 100,
        downloaded,
        total: total_size,
        speed: "Extracting...".to_string(),
    });

    // Extract the MDX file from zip
    extract_mdx_from_zip(&zip_path, config_dir)?;

    // Cleanup temp files
    let _ = fs::remove_file(&zip_path);
    let _ = fs::remove_dir(&temp_dir);

    Ok(())
}

fn format_speed(bytes_per_sec: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;

    if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.0} KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
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
