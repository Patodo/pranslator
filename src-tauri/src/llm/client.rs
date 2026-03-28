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

pub async fn translate_text(text: &str, prompt: &str, settings: &LlmSettings) -> Result<String> {
    // Build client without system proxy — proxy is only used for dictionary downloads
    let client = Client::builder().no_proxy().build()?;

    // Replace {{text}} placeholder with actual text and send as user message
    let prompt = prompt.replace("{{text}}", text);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_settings(api_base: &str) -> LlmSettings {
        LlmSettings {
            api_key: "test-key".to_string(),
            api_base: api_base.to_string(),
            model: "gpt-4o-mini".to_string(),
            system_prompt: String::new(),
        }
    }

    #[tokio::test]
    async fn test_translate_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .match_header("Authorization", "Bearer test-key")
            .match_header("Content-Type", "application/json")
            .with_status(200)
            .with_body(
                r#"{"choices":[{"message":{"role":"assistant","content":"你好世界"}}]}"#,
            )
            .create_async()
            .await;

        let settings = mock_settings(&server.url());
        let result = translate_text("hello world", "Translate: {{text}}", &settings).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "你好世界");
    }

    #[tokio::test]
    async fn test_translate_api_error() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .with_status(401)
            .with_body(r#"{"error":{"message":"Invalid API key"}}"#)
            .create_async()
            .await;

        let settings = mock_settings(&server.url());
        let result = translate_text("hello", "Translate: {{text}}", &settings).await;

        mock.assert_async().await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("API request failed"),
            "Expected API error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_translate_no_choices() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_body(r#"{"choices":[]}"#)
            .create_async()
            .await;

        let settings = mock_settings(&server.url());
        let result = translate_text("hello", "Translate: {{text}}", &settings).await;

        mock.assert_async().await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("No translation result"),
            "Expected no-result error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_translate_prompt_substitution() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .match_body(r#"{"model":"gpt-4o-mini","messages":[{"role":"user","content":"Translate this: hello world"}],"stream":false}"#)
            .with_status(200)
            .with_body(
                r#"{"choices":[{"message":{"role":"assistant","content":"翻译结果"}}]}"#,
            )
            .create_async()
            .await;

        let settings = mock_settings(&server.url());
        let result = translate_text("hello world", "Translate this: {{text}}", &settings).await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_translate_sends_correct_model() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .match_body(r#"{"model":"gpt-4","messages":[{"role":"user","content":"Translate: test"}],"stream":false}"#)
            .with_status(200)
            .with_body(
                r#"{"choices":[{"message":{"role":"assistant","content":"ok"}}]}"#,
            )
            .create_async()
            .await;

        let mut settings = mock_settings(&server.url());
        settings.model = "gpt-4".to_string();
        let result = translate_text("test", "Translate: {{text}}", &settings).await;

        mock.assert_async().await;
        assert!(result.is_ok());
    }
}
