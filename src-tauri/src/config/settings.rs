use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    pub api_key: String,
    pub api_base: String,
    pub model: String,
}

impl Default for LlmSettings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub llm: LlmSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            llm: LlmSettings::default(),
        }
    }
}

impl Settings {
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("pranslator");
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
            let settings: Settings = toml::from_str(&content)?;
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
