use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::config::Settings;
use crate::constants;

#[tauri::command]
pub fn validate_shortcut(shortcut: String) -> Result<(), String> {
    if shortcut.trim().is_empty() {
        return Err("Shortcut cannot be empty".to_string());
    }
    shortcut
        .parse::<Shortcut>()
        .map(|_| ())
        .map_err(|e| format!("Invalid shortcut: {}", e))
}

#[tauri::command]
pub fn get_config(settings: State<'_, Mutex<Settings>>) -> Result<Settings, String> {
    let settings = settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn set_config(
    new_settings: Settings,
    settings: State<'_, Mutex<Settings>>,
    app: AppHandle,
) -> Result<(), String> {
    let mut current_settings = settings.lock().map_err(|e| e.to_string())?;

    // Check if shortcut changed
    let shortcut_changed = current_settings.shortcuts.toggle_window != new_settings.shortcuts.toggle_window;

    // Unregister old shortcut if changed
    if shortcut_changed {
        if let Ok(old_shortcut) = current_settings.shortcuts.toggle_window.parse::<Shortcut>() {
            let _ = app.global_shortcut().unregister(old_shortcut);
        }
    }

    // Update settings
    *current_settings = new_settings.clone();
    current_settings.save().map_err(|e| e.to_string())?;

    // Register new shortcut if changed
    if shortcut_changed {
        let new_shortcut = new_settings
            .shortcuts
            .toggle_window
            .parse::<Shortcut>()
            .map_err(|e| format!("Invalid shortcut: {}", e))?;
        app.global_shortcut()
            .register(new_shortcut)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(constants::MAIN_WINDOW) {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}
