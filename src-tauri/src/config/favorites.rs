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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TestEnv;

    #[test]
    fn test_add_creates_item_with_id_and_timestamp() {
        let _env = TestEnv::new();

        let item = Favorites::add("hello".to_string(), "你好".to_string())
            .expect("add should succeed");

        assert!(!item.id.is_empty(), "id should be a non-empty UUID");
        assert!(Uuid::parse_str(&item.id).is_ok(), "id should be valid UUID");
        assert!(item.created_at > 0, "created_at should be a valid timestamp");
        assert_eq!(item.original_text, "hello");
        assert_eq!(item.translated_text, "你好");
    }

    #[test]
    fn test_add_newest_first() {
        let _env = TestEnv::new();

        Favorites::add("first".to_string(), "第一".to_string()).unwrap();
        Favorites::add("second".to_string(), "第二".to_string()).unwrap();
        Favorites::add("third".to_string(), "第三".to_string()).unwrap();

        let items = Favorites::get_all().expect("get_all should succeed");

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].original_text, "third");
        assert_eq!(items[1].original_text, "second");
        assert_eq!(items[2].original_text, "first");
    }

    #[test]
    fn test_delete_removes_target_only() {
        let _env = TestEnv::new();

        let a = Favorites::add("a".to_string(), "A".to_string()).unwrap();
        let b = Favorites::add("b".to_string(), "B".to_string()).unwrap();
        let c = Favorites::add("c".to_string(), "C".to_string()).unwrap();

        Favorites::delete(b.id.clone()).expect("delete should succeed");

        let items = Favorites::get_all().expect("get_all should succeed");
        assert_eq!(items.len(), 2);
        let ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains(&a.id.as_str()));
        assert!(ids.contains(&c.id.as_str()));
        assert!(!ids.contains(&b.id.as_str()));
    }

    #[test]
    fn test_delete_nonexistent_is_noop() {
        let _env = TestEnv::new();

        Favorites::add("keep".to_string(), "保留".to_string()).unwrap();
        Favorites::delete("nonexistent-uuid".to_string()).expect("delete should not error");

        let items = Favorites::get_all().expect("get_all should succeed");
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_load_missing_file_returns_empty() {
        let _env = TestEnv::new();

        let items = Favorites::get_all().expect("get_all should succeed");
        assert!(items.is_empty());
    }

    #[test]
    fn test_save_load_roundtrip() {
        let _env = TestEnv::new();

        let item = Favorites::add("hello".to_string(), "你好".to_string()).unwrap();

        // Simulate app restart by loading fresh
        let items = Favorites::get_all().expect("get_all should succeed");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, item.id);
        assert_eq!(items[0].original_text, "hello");
        assert_eq!(items[0].translated_text, "你好");
    }

    #[test]
    fn test_load_corrupted_json_returns_empty() {
        let _env = TestEnv::new();

        let path = Favorites::favorites_path().unwrap();
        fs::write(&path, "{corrupted json{{{").expect("write should succeed");

        let favorites = Favorites::load().expect("load should not panic");
        assert!(favorites.items.is_empty());
    }
}
