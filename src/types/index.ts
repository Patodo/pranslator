export interface LlmSettings {
  api_key: string;
  api_base: string;
  model: string;
}

export interface ShortcutSettings {
  toggle_window: string;
}

export interface Settings {
  llm: LlmSettings;
  shortcuts: ShortcutSettings;
}

export interface TranslateRequest {
  text: string;
}

export interface TranslateResponse {
  translated_text: string;
}
