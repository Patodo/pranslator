use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::LlmSettings;

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Debug, Deserialize)]
struct ChatMessageContent {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

pub async fn translate_text(text: &str, settings: &LlmSettings) -> Result<String> {
    let client = Client::new();

    // Replace {{text}} placeholder with actual text and send as user message
    let prompt = settings.system_prompt.replace("{{text}}", text);

    let request = ChatRequest {
        model: settings.model.clone(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }],
        stream: false,
    };

    let url = format!("{}/chat/completions", settings.api_base);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("API request failed: {}", error_text);
    }

    let chat_response: ChatResponse = response.json().await?;

    let translated_text = chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow::anyhow!("No translation result"))?;

    Ok(translated_text)
}
