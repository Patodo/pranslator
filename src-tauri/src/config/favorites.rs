// Favorites storage module
// Stores favorite translations in JSON format

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteItem {
    pub id: String,
    pub original_text: String,
    pub translated_text: String,
    pub created_at: i64, // Unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Favorites {
    pub items: Vec<FavoriteItem>,
}

impl Favorites {
    pub fn config_dir() -> Result<PathBuf> {
        // Use same config directory as settings
        if let Ok(custom_path) = std::env::var("PRANSLATOR_CONFIG_PATH") {
            let config_dir = PathBuf::from(custom_path);
            fs::create_dir_all(&config_dir)?;
            return Ok(config_dir);
        }

        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let config_dir = home.join(".config").join("pranslator");
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir)
    }

    pub fn favorites_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("favorites.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::favorites_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let favorites: Favorites = serde_json::from_str(&content).unwrap_or_default();
            Ok(favorites)
        } else {
            Ok(Favorites::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::favorites_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn add(original_text: String, translated_text: String) -> Result<FavoriteItem> {
        let mut favorites = Self::load()?;

        let item = FavoriteItem {
            id: Uuid::new_v4().to_string(),
            original_text,
            translated_text,
            created_at: chrono::Utc::now().timestamp(),
        };

        // Insert at the beginning (newest first)
        favorites.items.insert(0, item.clone());
        favorites.save()?;

        Ok(item)
    }

    pub fn delete(id: String) -> Result<()> {
        let mut favorites = Self::load()?;
        favorites.items.retain(|item| item.id != id);
        favorites.save()?;
        Ok(())
    }

    pub fn get_all() -> Result<Vec<FavoriteItem>> {
        let favorites = Self::load()?;
        Ok(favorites.items)
    }
}
