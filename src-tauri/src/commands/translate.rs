use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::config::{LlmSettings, Settings};
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

/// Resolved translation route — either a dictionary hit or an LLM call config.
#[derive(Debug)]
pub enum TranslationRoute {
    Dictionary(String),
    Llm { prompt: String },
}

/// Pure routing logic: decides whether to use dictionary or LLM, and which prompt.
/// This is the core business rule extracted for testability.
pub fn resolve_translation_route(
    text: &str,
    mode: Option<&str>,
    dict: Option<&Dictionary>,
    llm_settings: &LlmSettings,
) -> Result<TranslationRoute, String> {
    // For word mode, try dictionary first
    if mode == Some("word") {
        if let Some(dict) = dict {
            if let Some(result) = dict.lookup(text) {
                return Ok(TranslationRoute::Dictionary(result));
            }
        }
    }

    if llm_settings.api_key.is_empty() {
        return Err("API key not configured. Please set your API key in settings.".to_string());
    }

    let prompt = if mode == Some("word") {
        WORD_TRANSLATION_PROMPT.to_string()
    } else {
        llm_settings.system_prompt.clone()
    };

    Ok(TranslationRoute::Llm { prompt })
}

#[tauri::command]
pub async fn translate(
    request: TranslateRequest,
    settings: State<'_, Mutex<Settings>>,
    dict: State<'_, Mutex<Option<Dictionary>>>,
) -> Result<TranslateResponse, String> {
    let (route, llm_settings) = {
        let settings = settings.lock().map_err(|e| e.to_string())?;
        let dict_guard = dict.lock().map_err(|e| e.to_string())?;
        let route = resolve_translation_route(
            &request.text,
            request.mode.as_deref(),
            dict_guard.as_ref(),
            &settings.llm,
        )?;
        (route, settings.llm.clone())
    };

    match route {
        TranslationRoute::Dictionary(result) => Ok(TranslateResponse {
            translated_text: result,
            source: Some("dictionary".to_string()),
        }),
        TranslationRoute::Llm { prompt } => {
            let translated_text =
                client::translate_text(&request.text, &prompt, &llm_settings)
                    .await
                    .map_err(|e| e.to_string())?;

            Ok(TranslateResponse {
                translated_text,
                source: Some("llm".to_string()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LlmSettings;
    use crate::constants::{DEFAULT_SYSTEM_PROMPT, WORD_TRANSLATION_PROMPT};
    use std::collections::HashMap;

    fn test_llm_settings() -> LlmSettings {
        LlmSettings {
            api_key: "sk-test".to_string(),
            api_base: "https://api.example.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
        }
    }

    fn test_dict_with_word(word: &str, definition: &str) -> Dictionary {
        let mut entries = HashMap::new();
        entries.insert(word.to_lowercase(), definition.to_string());
        Dictionary::from_entries(entries)
    }

    #[test]
    fn test_word_mode_dict_hit() {
        let dict = test_dict_with_word("hello", "你好，用于问候");
        let settings = test_llm_settings();

        let route = resolve_translation_route("hello", Some("word"), Some(&dict), &settings);

        match route.unwrap() {
            TranslationRoute::Dictionary(text) => assert!(text.contains("你好")),
            other => panic!("Expected Dictionary, got: {:?}", route_name(&other)),
        }
    }

    #[test]
    fn test_word_mode_dict_miss_falls_to_llm() {
        let dict = test_dict_with_word("hello", "你好");
        let settings = test_llm_settings();

        let route = resolve_translation_route("goodbye", Some("word"), Some(&dict), &settings);

        match route.unwrap() {
            TranslationRoute::Llm { prompt } => assert_eq!(prompt, WORD_TRANSLATION_PROMPT),
            other => panic!("Expected Llm, got: {:?}", route_name(&other)),
        }
    }

    #[test]
    fn test_word_mode_no_dict_uses_word_prompt() {
        let settings = test_llm_settings();

        let route = resolve_translation_route("hello", Some("word"), None, &settings);

        match route.unwrap() {
            TranslationRoute::Llm { prompt } => assert_eq!(prompt, WORD_TRANSLATION_PROMPT),
            other => panic!("Expected Llm, got: {:?}", route_name(&other)),
        }
    }

    #[test]
    fn test_non_word_mode_uses_user_prompt() {
        let settings = test_llm_settings();

        let route = resolve_translation_route("hello world", None, None, &settings);

        match route.unwrap() {
            TranslationRoute::Llm { prompt } => assert_eq!(prompt, settings.system_prompt),
            other => panic!("Expected Llm, got: {:?}", route_name(&other)),
        }
    }

    #[test]
    fn test_empty_api_key_rejected() {
        let mut settings = test_llm_settings();
        settings.api_key = String::new();

        let route = resolve_translation_route("hello", None, None, &settings);

        assert!(route.is_err());
        let err = route.unwrap_err();
        assert!(
            err.contains("API key not configured"),
            "Expected API key error, got: {err}"
        );
    }

    #[test]
    fn test_word_mode_dict_hit_case_insensitive() {
        let dict = test_dict_with_word("hello", "你好");
        let settings = test_llm_settings();

        let route = resolve_translation_route("Hello", Some("word"), Some(&dict), &settings);

        match route.unwrap() {
            TranslationRoute::Dictionary(text) => assert_eq!(text, "你好"),
            other => panic!("Expected Dictionary, got: {:?}", route_name(&other)),
        }
    }

    fn route_name(route: &TranslationRoute) -> &'static str {
        match route {
            TranslationRoute::Dictionary(_) => "Dictionary",
            TranslationRoute::Llm { .. } => "Llm",
        }
    }
}
