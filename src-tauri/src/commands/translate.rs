use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::config::Settings;
use crate::llm::client;

#[derive(Debug, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct TranslateResponse {
    pub translated_text: String,
}

#[tauri::command]
pub async fn translate(
    request: TranslateRequest,
    settings: State<'_, Mutex<Settings>>,
) -> Result<TranslateResponse, String> {
    let llm_settings = {
        let settings = settings.lock().map_err(|e| e.to_string())?;
        settings.llm.clone()
    };

    if llm_settings.api_key.is_empty() {
        return Err("API key not configured. Please set your API key in settings.".to_string());
    }

    let translated_text = client::translate_text(&request.text, &llm_settings)
        .await
        .map_err(|e| e.to_string())?;

    Ok(TranslateResponse { translated_text })
}
