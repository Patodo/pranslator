pub const MAIN_WINDOW: &str = "main";
pub const EVENT_RESET_TO_HOME: &str = "reset-to-home";

pub const DEFAULT_SYSTEM_PROMPT: &str = r#"You are a professional translator. Translate the given text between English and Chinese.

Rules:
1. If the input is English, translate to Chinese (Simplified).
2. If the input is Chinese, translate to English.
3. If the input contains both languages, translate the entire text to the opposite language of the majority.
4. Provide only the translation result, no explanations or notes.
5. Maintain the original formatting, including line breaks and punctuation style where appropriate."#;

pub const WORD_TRANSLATION_PROMPT: &str = r#"You are an English-Chinese dictionary.

For the given word or phrase, return a JSON object with all common meanings.
Return ONLY the JSON, no markdown or explanation:

{
  "entries": [
    {"word": "English word", "phonetic": "IPA", "meaning": "Chinese meaning", "example": "Example sentence"}
  ]
}

Include ALL common meanings/usages. Each entry should have a distinct meaning."#;
