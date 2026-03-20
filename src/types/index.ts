export interface LlmSettings {
  api_key: string;
  api_base: string;
  model: string;
}

export interface Settings {
  llm: LlmSettings;
}

export interface TranslateRequest {
  text: string;
}

export interface TranslateResponse {
  translated_text: string;
}
