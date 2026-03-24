// Type definitions for frontend-backend communication
// Corresponds to Rust types in src-tauri/src/config/settings.rs

export interface LlmSettings {
  // Corresponds to Rust: LlmSettings
  api_key: string;
  api_base: string;
  model: string;
  system_prompt: string;
}

export interface ShortcutSettings {
  // Corresponds to Rust: ShortcutSettings
  toggle_window: string;
}

export interface Settings {
  // Corresponds to Rust: Settings
  llm: LlmSettings;
  shortcuts: ShortcutSettings;
}

export interface TranslateRequest {
  // Corresponds to Rust: TranslateRequest in commands/translate.rs
  text: string;
}

export interface TranslateResponse {
  // Corresponds to Rust: TranslateResponse in commands/translate.rs
  translated_text: string;
}

export interface FavoriteItem {
  // Corresponds to Rust: FavoriteItem in config/favorites.rs
  id: string;
  original_text: string;
  translated_text: string;
  created_at: number; // Unix timestamp
}
