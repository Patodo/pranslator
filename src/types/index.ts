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

export interface WordEntry {
  word: string;
  phonetic: string;
  meaning: string;
  example: string;
}

export interface WordResponse {
  entries: WordEntry[];
}

export type TranslationMode = 'normal' | 'word';

export interface TranslateRequest {
  // Corresponds to Rust: TranslateRequest in commands/translate.rs
  text: string;
  mode?: TranslationMode;
}

export interface TranslateResponse {
  // Corresponds to Rust: TranslateResponse in commands/translate.rs
  translated_text: string;
  source?: 'dictionary' | 'llm';
}

export interface DictionaryStatus {
  downloaded: boolean;
  downloading: boolean;
  progress: number; // 0-100
  fileSize?: string;
}

export interface FavoriteItem {
  // Corresponds to Rust: FavoriteItem in config/favorites.rs
  id: string;
  original_text: string;
  translated_text: string;
  created_at: number; // Unix timestamp
}
