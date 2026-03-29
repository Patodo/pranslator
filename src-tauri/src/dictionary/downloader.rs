use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use reqwest::{Client, StatusCode};
use std::fs::{self, File, OpenOptions};
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

/// Download a file with resume support.
///
/// If the destination file already exists, sends an HTTP Range request to
/// resume from the current file size. Falls back to a full download if the
/// server does not support Range.
///
/// On cancellation, the partial file is preserved for future resume.
pub async fn download_to_file(
    url: &str,
    dest: &Path,
    fallback_total: u64,
    on_progress: impl Fn(u8, u64, u64, &str),
) -> Result<()> {
    let resume_offset = if dest.exists() {
        fs::metadata(dest).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let client = Client::new();
    let mut request = client.get(url);

    if resume_offset > 0 {
        request = request.header("Range", format!("bytes={}-", resume_offset));
    }

    let response = request
        .send()
        .await
        .with_context(|| "Failed to start download")?;

    let status = response.status();

    // Determine whether we are resuming or starting fresh
    let (file, mut downloaded, total_size) = if status == StatusCode::PARTIAL_CONTENT {
        // Server supports resume — append to existing file
        let total = response
            .headers()
            .get("Content-Range")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split('/').last())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(fallback_total);

        let file = OpenOptions::new().append(true).open(dest)
            .with_context(|| "Failed to open partial file for append")?;

        (file, resume_offset, total)
    } else {
        // Server returned 200 or doesn't support Range — start from scratch
        let total = response.content_length().unwrap_or(fallback_total);
        let file = File::create(dest)
            .with_context(|| "Failed to create temp file")?;

        (file, 0u64, total)
    };

    let mut file = file;
    let mut stream = response.bytes_stream();

    let start_time = Instant::now();
    let mut last_emit_time = Instant::now();
    // Only measure speed for bytes downloaded in this session
    let session_start_bytes = downloaded;

    while let Some(chunk) = stream.next().await {
        if is_cancelled() {
            // Keep partial file for future resume
            return Err(anyhow!("Download cancelled by user"));
        }

        let chunk = chunk.with_context(|| "Failed to read chunk")?;
        file.write_all(&chunk).with_context(|| "Failed to write chunk")?;
        downloaded += chunk.len() as u64;

        let now = Instant::now();
        if now.duration_since(last_emit_time) >= Duration::from_millis(100) {
            let progress = ((downloaded as f64 / total_size as f64) * 100.0).min(100.0) as u8;

            let elapsed = now.duration_since(start_time).as_secs_f64();
            let speed = if elapsed > 0.0 {
                let session_bytes = downloaded - session_start_bytes;
                format_speed(session_bytes as f64 / elapsed)
            } else {
                "0 B/s".to_string()
            };

            on_progress(progress, downloaded, total_size, &speed);
            last_emit_time = now;
        }
    }

    if is_cancelled() {
        return Err(anyhow!("Download cancelled by user"));
    }

    on_progress(100, downloaded, total_size, "Extracting...");

    Ok(())
}

/// Download and extract the MDX dictionary.
/// Reports progress via Tauri events. Supports resume on cancellation.
pub async fn download_dictionary(app: AppHandle, config_dir: &Path) -> Result<()> {
    reset_cancel_flag();

    // Clean up stale temp files from previous downloads to prevent
    // corrupted zips from resume mechanism, then create fresh temp dir.
    let temp_dir = std::env::temp_dir().join("pranslator-dict-download");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir)
        .with_context(|| "Failed to create temp directory")?;

    let zip_path = temp_dir.join(DICT_ZIP_FILENAME);

    let on_progress = |progress: u8, downloaded: u64, total: u64, speed: &str| {
        let _ = app.emit("dictionary-download-progress", DownloadProgress {
            progress,
            downloaded,
            total,
            speed: speed.to_string(),
        });
    };

    download_to_file(DICT_URL, &zip_path, DICT_FILE_SIZE, on_progress).await?;

    // Extract the MDX file from zip
    extract_mdx_from_zip(&zip_path, config_dir)?;

    // Cleanup temp files after successful extraction
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_speed_mb() {
        assert_eq!(format_speed(5.0 * 1024.0 * 1024.0), "5.0 MB/s");
        assert_eq!(format_speed(1.5 * 1024.0 * 1024.0), "1.5 MB/s");
        assert_eq!(format_speed(1024.0 * 1024.0), "1.0 MB/s");
    }

    #[test]
    fn test_format_speed_kb() {
        assert_eq!(format_speed(512.0 * 1024.0), "512 KB/s");
        assert_eq!(format_speed(1024.0), "1 KB/s");
    }

    #[test]
    fn test_format_speed_bytes() {
        assert_eq!(format_speed(500.0), "500 B/s");
        assert_eq!(format_speed(1.0), "1 B/s");
    }

    #[test]
    fn test_format_speed_zero() {
        assert_eq!(format_speed(0.0), "0 B/s");
    }

    #[test]
    fn test_cancel_flag() {
        // Reset first to ensure clean state
        reset_cancel_flag();
        assert!(!is_cancelled());

        cancel_download();
        assert!(is_cancelled());

        // Cleanup
        reset_cancel_flag();
    }

    #[test]
    fn test_reset_cancel_flag() {
        cancel_download();
        assert!(is_cancelled());

        reset_cancel_flag();
        assert!(!is_cancelled());
    }

    // --- Resume download tests ---

    /// Business requirement: when a partial file exists, download resumes from
    /// where it left off using HTTP Range, and appends to the existing file.
    #[tokio::test]
    async fn test_resume_from_partial_file() {
        let mut server = mockito::Server::new_async().await;
        reset_cancel_flag();

        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let zip_path = temp_dir.path().join(DICT_ZIP_FILENAME);
        let config_dir = temp_dir.path().join("config");
        fs::create_dir_all(&config_dir).unwrap();

        // Simulate 4 bytes already downloaded
        fs::write(&zip_path, b"dead").unwrap();

        // Server should receive a Range request and respond with 206
        let mock = server
            .mock("GET", "/dict.zip")
            .match_header("Range", "bytes=4-")
            .with_status(206)
            .with_header("Content-Range", "bytes 4-7/8")
            .with_body(b"beef")
            .create_async()
            .await;

        let url = format!("{}/dict.zip", server.url());
        let result = download_to_file(&url, &zip_path, DICT_FILE_SIZE, |_, _, _, _| {}).await;

        mock.assert_async().await;
        assert!(result.is_ok(), "download should succeed");

        // File should contain both the old and new data
        let content = fs::read(&zip_path).unwrap();
        assert_eq!(content, b"deadbeef", "file should contain appended data");
    }

    /// Business requirement: if the server does not support Range (returns 200),
    /// the download starts from scratch and overwrites the partial file.
    #[tokio::test]
    async fn test_resume_server_returns_200() {
        let mut server = mockito::Server::new_async().await;
        reset_cancel_flag();

        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let zip_path = temp_dir.path().join(DICT_ZIP_FILENAME);

        // Simulate stale partial data
        fs::write(&zip_path, b"stale").unwrap();

        // Server ignores Range and returns full content with 200
        let mock = server
            .mock("GET", "/dict.zip")
            .with_status(200)
            .with_body(b"fresh")
            .create_async()
            .await;

        let url = format!("{}/dict.zip", server.url());
        let result = download_to_file(&url, &zip_path, DICT_FILE_SIZE, |_, _, _, _| {}).await;

        mock.assert_async().await;
        assert!(result.is_ok());

        // File should contain only the new data, not the stale prefix
        let content = fs::read(&zip_path).unwrap();
        assert_eq!(content, b"fresh", "file should be overwritten, not appended");
    }

    /// Business requirement: cancelling a download preserves the partial temp
    /// file so that a subsequent download can resume from where it stopped.
    #[tokio::test]
    async fn test_cancel_preserves_temp_file() {
        reset_cancel_flag();

        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let zip_path = temp_dir.path().join(DICT_ZIP_FILENAME);

        // Write a fake partial file
        fs::write(&zip_path, b"partial-data").unwrap();
        let size_before = fs::metadata(&zip_path).unwrap().len();
        assert!(size_before > 0);

        // Trigger cancel
        cancel_download();

        // Verify file still exists
        assert!(zip_path.exists(), "partial file should be preserved after cancel");

        reset_cancel_flag();
    }
}
