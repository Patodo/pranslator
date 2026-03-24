// Type definitions for settings
// Corresponds to TypeScript types in src/types/index.ts

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::constants::DEFAULT_SYSTEM_PROMPT;

fn default_system_prompt() -> String {
    DEFAULT_SYSTEM_PROMPT.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    // Corresponds to TypeScript: LlmSettings
    pub api_key: String,
    pub api_base: String,
    pub model: String,
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,
}

impl Default for LlmSettings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutSettings {
    // Corresponds to TypeScript: ShortcutSettings
    pub toggle_window: String,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        Self {
            toggle_window: "Alt+Shift+T".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // Corresponds to TypeScript: Settings
    pub llm: LlmSettings,
    pub shortcuts: ShortcutSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            llm: LlmSettings::default(),
            shortcuts: ShortcutSettings::default(),
        }
    }
}

impl Settings {
    pub fn config_dir() -> Result<PathBuf> {
        // 优先读取环境变量（开发模式）
        if let Ok(custom_path) = std::env::var("PRANSLATOR_CONFIG_PATH") {
            let config_dir = PathBuf::from(custom_path);
            fs::create_dir_all(&config_dir)?;
            return Ok(config_dir);
        }

        // 默认路径: ~/.config/pranslator/
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let config_dir = home.join(".config").join("pranslator");
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("settings.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            // Try to parse, fallback to default for missing fields
            let settings: Settings = toml::from_str(&content).unwrap_or_else(|_| {
                // Merge with defaults if parsing fails (e.g., missing new fields)
                let defaults = Settings::default();
                Settings {
                    llm: toml::from_str(&content)
                        .map(|s: LlmSettings| s)
                        .unwrap_or(defaults.llm),
                    shortcuts: defaults.shortcuts,
                }
            });
            Ok(settings)
        } else {
            let settings = Settings::default();
            settings.save()?;
            Ok(settings)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }
}
