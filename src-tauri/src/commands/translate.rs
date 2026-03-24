use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::config::Settings;
use crate::constants::WORD_TRANSLATION_PROMPT;
use crate::dictionary::reader::Dictionary;
use crate::llm::client;

#[derive(Debug, Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    pub mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TranslateResponse {
    pub translated_text: String,
    pub source: Option<String>,
}

#[tauri::command]
pub async fn translate(
    request: TranslateRequest,
    settings: State<'_, Mutex<Settings>>,
    dict: State<'_, Mutex<Option<Dictionary>>>,
) -> Result<TranslateResponse, String> {
    // For word mode, try dictionary first
    if request.mode.as_deref() == Some("word") {
        // Try dictionary lookup
        if let Some(dict) = dict.lock().map_err(|e| e.to_string())?.as_ref() {
            if let Some(result) = dict.lookup(&request.text) {
                return Ok(TranslateResponse {
                    translated_text: result,
                    source: Some("dictionary".to_string()),
                });
            }
        }
        // Fall through to LLM if dictionary lookup failed
    }

    let llm_settings = {
        let settings = settings.lock().map_err(|e| e.to_string())?;
        settings.llm.clone()
    };

    if llm_settings.api_key.is_empty() {
        return Err("API key not configured. Please set your API key in settings.".to_string());
    }

    // Determine which prompt to use based on mode
    let prompt = if request.mode.as_deref() == Some("word") {
        WORD_TRANSLATION_PROMPT.to_string()
    } else {
        llm_settings.system_prompt.clone()
    };

    let translated_text = client::translate_text(&request.text, &prompt, &llm_settings)
        .await
        .map_err(|e| e.to_string())?;

    Ok(TranslateResponse {
        translated_text,
        source: Some("llm".to_string()),
    })
}
