use std::sync::Mutex;
use tauri::State;

use crate::config::Settings;

#[tauri::command]
pub fn get_config(settings: State<'_, Mutex<Settings>>) -> Result<Settings, String> {
    let settings = settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn set_config(
    new_settings: Settings,
    settings: State<'_, Mutex<Settings>>,
) -> Result<(), String> {
    let mut current_settings = settings.lock().map_err(|e| e.to_string())?;
    *current_settings = new_settings;
    current_settings.save().map_err(|e| e.to_string())?;
    Ok(())
}
