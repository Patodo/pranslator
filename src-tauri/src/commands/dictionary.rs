use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

use crate::dictionary::{self, reader::Dictionary};
use crate::dictionary::is_dict_downloaded;

#[derive(Debug, Serialize)]
pub struct DictionaryStatus {
    pub downloaded: bool,
    pub downloading: bool,
    pub progress: u8,
    pub file_size: Option<String>,
}

impl Default for DictionaryStatus {
    fn default() -> Self {
        DictionaryStatus {
            downloaded: false,
            downloading: false,
            progress: 0,
            file_size: None,
        }
    }
}

#[tauri::command]
pub fn get_dictionary_status(
    app: AppHandle,
    dict_state: State<'_, Mutex<Option<Dictionary>>>,
) -> DictionaryStatus {
    let config_dir = app
        .path()
        .app_config_dir()
        .expect("Failed to get config dir");

    let downloaded = is_dict_downloaded(&config_dir);

    // Get file size if downloaded
    let file_size = if downloaded {
        std::fs::metadata(config_dir.join(dictionary::DICT_FILENAME))
            .ok()
            .map(|m| format_bytes(m.len()))
    } else {
        None
    };

    // `dict_state` is kept in the signature so Tauri injects the managed
    // state, but the downloaded flag now purely reflects file existence.
    let _ = dict_state;

    DictionaryStatus {
        downloaded,
        downloading: false, // This is tracked on frontend
        progress: 0,
        file_size,
    }
}

#[tauri::command]
pub async fn download_dictionary(
    app: AppHandle,
    dict_state: State<'_, Mutex<Option<Dictionary>>>,
) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;

    // Download and extract
    dictionary::downloader::download_dictionary(app.clone(), &config_dir)
        .await
        .map_err(|e| e.to_string())?;

    // Load the dictionary into memory
    let dict_path = dictionary::get_dict_path(&config_dir);
    let dict = Dictionary::open(&dict_path).map_err(|e| e.to_string())?;

    // Update state
    {
        let mut state = dict_state.lock().map_err(|e| e.to_string())?;
        *state = Some(dict);
    }

    Ok(())
}

#[tauri::command]
pub fn delete_dictionary(
    app: AppHandle,
    dict_state: State<'_, Mutex<Option<Dictionary>>>,
) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;

    // Delete the file
    dictionary::downloader::delete_dictionary(&config_dir)
        .map_err(|e| e.to_string())?;

    // Clear from memory
    {
        let mut state = dict_state.lock().map_err(|e| e.to_string())?;
        *state = None;
    }

    Ok(())
}

#[tauri::command]
pub fn cancel_dictionary_download() {
    dictionary::downloader::cancel_download();
}

/// Load dictionary at startup if it exists.
pub fn load_dictionary_if_exists(app: &AppHandle) -> Option<Dictionary> {
    let config_dir = app.path().app_config_dir().ok()?;

    if !is_dict_downloaded(&config_dir) {
        return None;
    }

    let dict_path = dictionary::get_dict_path(&config_dir);
    Dictionary::open(&dict_path).ok()
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
