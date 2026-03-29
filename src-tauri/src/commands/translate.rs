use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use tauri::{Manager, State};

use crate::config::{LlmSettings, Settings};
use crate::dictionary::{self, reader::Dictionary};
use crate::llm::client;

/// Structured word entry returned by dictionary lookups.
#[derive(Debug, Serialize)]
pub struct WordEntry {
    pub word: String,
    pub phonetic: String,
    pub meaning: String,
    pub example: String,
}

/// Word-mode response containing one or more dictionary entries.
#[derive(Debug, Serialize)]
pub struct WordResponse {
    pub entries: Vec<WordEntry>,
}

/// Strip HTML tags from a string, keeping only the text content.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    // Collapse consecutive whitespace
    let collapsed: String = result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    collapsed.trim().to_string()
}

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
    // Word mode: dictionary only, no LLM fallback
    if mode == Some("word") {
        let dict = dict.ok_or_else(|| {
            "Word mode requires offline dictionary. Please download it in Settings.".to_string()
        })?;
        return dict
            .lookup(text)
            .map(TranslationRoute::Dictionary)
            .ok_or_else(|| "Word not found in dictionary".to_string());
    }

    // Normal mode: always LLM
    if llm_settings.api_key.is_empty() {
        return Err("API key not configured. Please set your API key in settings.".to_string());
    }

    Ok(TranslationRoute::Llm {
        prompt: llm_settings.system_prompt.clone(),
    })
}

/// Ensure the dictionary is loaded into memory if the file exists on disk.
/// This handles the case where `Dictionary::open` failed at startup but the
/// file is present (e.g. downloaded in a previous session).
fn ensure_dict_loaded(
    dict: &Mutex<Option<Dictionary>>,
    config_dir: &Path,
) -> Result<(), String> {
    {
        let guard = dict.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Ok(());
        }
    }

    // File doesn't exist — nothing to load
    if !dictionary::is_dict_downloaded(config_dir) {
        return Ok(());
    }

    // Try to load
    let dict_path = dictionary::get_dict_path(config_dir);
    match Dictionary::open(&dict_path) {
        Ok(loaded) => {
            let mut guard = dict.lock().map_err(|e| e.to_string())?;
            *guard = Some(loaded);
            Ok(())
        }
        Err(e) => {
            // Auto-delete corrupted file so the user can re-download
            let _ = std::fs::remove_file(&dict_path);
            Err(format!(
                "Dictionary file was corrupted and has been removed: {}. \
                 Please re-download in Settings.",
                e
            ))
        }
    }
}

#[tauri::command]
pub async fn translate(
    app: tauri::AppHandle,
    request: TranslateRequest,
    settings: State<'_, Mutex<Settings>>,
    dict: State<'_, Mutex<Option<Dictionary>>>,
) -> Result<TranslateResponse, String> {
    // Lazy-load dictionary if file exists but isn't in memory
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    ensure_dict_loaded(&dict, &config_dir)?;

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
        TranslationRoute::Dictionary(raw) => {
            let plain = strip_html_tags(&raw);
            let response = WordResponse {
                entries: vec![WordEntry {
                    word: request.text.clone(),
                    phonetic: String::new(),
                    meaning: plain,
                    example: String::new(),
                }],
            };
            let translated_text =
                serde_json::to_string(&response).map_err(|e| e.to_string())?;
            Ok(TranslateResponse {
                translated_text,
                source: Some("dictionary".to_string()),
            })
        }
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
    use crate::constants::DEFAULT_SYSTEM_PROMPT;
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
    fn test_word_mode_dict_miss_returns_error() {
        let dict = test_dict_with_word("hello", "你好");
        let settings = test_llm_settings();

        let route = resolve_translation_route("goodbye", Some("word"), Some(&dict), &settings);

        assert!(route.is_err());
        let err = route.unwrap_err();
        assert!(
            err.contains("Word not found"),
            "Expected 'Word not found' error, got: {err}"
        );
    }

    #[test]
    fn test_word_mode_no_dict_returns_error() {
        let settings = test_llm_settings();

        let route = resolve_translation_route("hello", Some("word"), None, &settings);

        assert!(route.is_err());
        let err = route.unwrap_err();
        assert!(
            err.contains("offline dictionary"),
            "Expected dictionary required error, got: {err}"
        );
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

    // --- strip_html_tags tests ---

    #[test]
    fn test_strip_html_basic_tags() {
        assert_eq!(strip_html_tags("<b>hello</b>"), "hello");
        assert_eq!(
            strip_html_tags("<div>你好</div>"),
            "你好"
        );
    }

    #[test]
    fn test_strip_html_nested_tags() {
        assert_eq!(
            strip_html_tags("<ul><li><b>hello</b></li></ul>"),
            "hello"
        );
    }

    #[test]
    fn test_strip_html_no_tags() {
        assert_eq!(strip_html_tags("plain text"), "plain text");
    }

    #[test]
    fn test_strip_html_empty() {
        assert_eq!(strip_html_tags(""), "");
    }

    #[test]
    fn test_strip_html_collapses_whitespace() {
        assert_eq!(
            strip_html_tags("<p>  hello   world  </p>"),
            "hello world"
        );
    }

    #[test]
    fn test_strip_html_preserves_entities() {
        // HTML entities are not decoded, just tags stripped
        assert_eq!(strip_html_tags("a &amp; b"), "a &amp; b");
    }

    // --- WordResponse JSON format tests ---

    #[test]
    fn test_word_mode_dict_hit_returns_word_response_json() {
        let dict = test_dict_with_word(
            "hello",
            "<ul><li>/həˈloʊ/</li><li>used as a greeting</li></ul>",
        );
        let settings = test_llm_settings();

        let route =
            resolve_translation_route("hello", Some("word"), Some(&dict), &settings);

        match route.unwrap() {
            TranslationRoute::Dictionary(raw) => {
                // The raw value should be the HTML from the dictionary
                assert!(raw.contains("greeting"));

                // Verify strip_html_tags produces clean text
                let plain = strip_html_tags(&raw);
                assert!(!plain.contains('<'));
                assert!(plain.contains("greeting"));

                // Verify the WordResponse structure can be serialized
                let response = WordResponse {
                    entries: vec![WordEntry {
                        word: "hello".to_string(),
                        phonetic: String::new(),
                        meaning: plain,
                        example: String::new(),
                    }],
                };
                let json = serde_json::to_string(&response).unwrap();
                let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
                assert_eq!(parsed["entries"][0]["word"], "hello");
                assert!(parsed["entries"][0]["meaning"].as_str().unwrap().contains("greeting"));
            }
            other => panic!("Expected Dictionary, got: {:?}", route_name(&other)),
        }
    }

    #[test]
    fn test_word_response_json_matches_frontend_type() {
        let response = WordResponse {
            entries: vec![WordEntry {
                word: "hello".to_string(),
                phonetic: "/həˈloʊ/".to_string(),
                meaning: "used as a greeting".to_string(),
                example: "Hello, how are you?".to_string(),
            }],
        };
        let json = serde_json::to_string(&response).unwrap();

        // Verify the JSON structure matches what the frontend expects:
        // { entries: [{ word, phonetic, meaning, example }] }
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let entries = parsed["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["word"], "hello");
        assert_eq!(entries[0]["phonetic"], "/həˈloʊ/");
        assert_eq!(entries[0]["meaning"], "used as a greeting");
        assert_eq!(entries[0]["example"], "Hello, how are you?");
    }

    #[test]
    fn test_word_response_multiple_entries() {
        let response = WordResponse {
            entries: vec![
                WordEntry {
                    word: "hello".to_string(),
                    phonetic: String::new(),
                    meaning: "greeting".to_string(),
                    example: String::new(),
                },
                WordEntry {
                    word: "hello".to_string(),
                    phonetic: String::new(),
                    meaning: "an expression of surprise".to_string(),
                    example: String::new(),
                },
            ],
        };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["entries"].as_array().unwrap().len(), 2);
    }

    // --- End-to-end pipeline tests with fake dictionary ---

    /// Simulate the full word-mode pipeline:
    /// inject fake dict → resolve route → build WordResponse → parse JSON
    /// This catches regressions where the frontend can't parse the result.
    #[test]
    fn test_word_pipeline_dict_hit_produces_valid_frontend_json() {
        let dict = test_dict_with_word(
            "hello",
            "<ul><li>/həˈloʊ/</li><li>used as a greeting</li><li>Hello, world!</li></ul>",
        );
        let settings = test_llm_settings();

        // Step 1: resolve route (what the Tauri command does)
        let route = resolve_translation_route("hello", Some("word"), Some(&dict), &settings);
        let TranslationRoute::Dictionary(raw) = route.unwrap() else {
            panic!("Expected Dictionary route");
        };

        // Step 2: build WordResponse (what the Tauri command does)
        let plain = strip_html_tags(&raw);
        let response = WordResponse {
            entries: vec![WordEntry {
                word: "hello".to_string(),
                phonetic: String::new(),
                meaning: plain,
                example: String::new(),
            }],
        };
        let translated_text = serde_json::to_string(&response).unwrap();

        // Step 3: parse as frontend would (JSON.parse → WordResponse)
        let parsed: serde_json::Value = serde_json::from_str(&translated_text)
            .expect("Frontend JSON.parse should succeed");

        // Verify structure matches the TypeScript WordResponse interface
        let entries = parsed["entries"].as_array().expect("entries should be array");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["word"].as_str(), Some("hello"));
        assert!(entries[0]["meaning"].as_str().unwrap().contains("greeting"));
        assert!(entries[0]["meaning"].as_str().unwrap().contains("/həˈloʊ/"));
        // No HTML tags in meaning
        assert!(!entries[0]["meaning"].as_str().unwrap().contains('<'));
    }

    /// Simulate word-mode miss: dict loaded but word not present
    #[test]
    fn test_word_pipeline_dict_miss_returns_clear_error() {
        let dict = test_dict_with_word("apple", "苹果");
        let settings = test_llm_settings();

        let route = resolve_translation_route("banana", Some("word"), Some(&dict), &settings);

        let err = route.unwrap_err();
        assert!(
            err.contains("Word not found"),
            "Should explain word was not found, got: {err}"
        );
    }

    /// Simulate word-mode with no dictionary available
    #[test]
    fn test_word_pipeline_no_dict_returns_clear_error() {
        let settings = test_llm_settings();

        let route = resolve_translation_route("hello", Some("word"), None, &settings);

        let err = route.unwrap_err();
        assert!(
            err.contains("offline dictionary"),
            "Should mention dictionary requirement, got: {err}"
        );
    }

    /// Ensure dictionary lookup + strip_html_tags + JSON roundtrip
    /// works for Chinese-English content (the real ECDICT use case)
    #[test]
    fn test_word_pipeline_chinese_content_html_to_json() {
        let dict = test_dict_with_word(
            "apple",
            "<div><span class=\"phonic\">/ˈæp.əl/</span></div>\
             <div><font color=\"blue\">n.</font> 苹果</div>\
             <div>I ate an apple.</div>",
        );
        let settings = test_llm_settings();

        let route = resolve_translation_route("apple", Some("word"), Some(&dict), &settings);
        let TranslationRoute::Dictionary(raw) = route.unwrap() else {
            panic!("Expected Dictionary route");
        };

        let plain = strip_html_tags(&raw);
        let response = WordResponse {
            entries: vec![WordEntry {
                word: "apple".to_string(),
                phonetic: String::new(),
                meaning: plain,
                example: String::new(),
            }],
        };
        let json = serde_json::to_string(&response).unwrap();

        // Frontend parse
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let meaning = parsed["entries"][0]["meaning"].as_str().unwrap();

        // All content preserved, no HTML
        assert!(meaning.contains("/ˈæp.əl/"));
        assert!(meaning.contains("苹果"));
        assert!(meaning.contains("apple"));
        assert!(!meaning.contains('<'));
        assert!(!meaning.contains('>'));
    }
}
