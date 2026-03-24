use crate::config::{FavoriteItem, Favorites};

#[tauri::command]
pub fn get_favorites() -> Result<Vec<FavoriteItem>, String> {
    Favorites::get_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_favorite(original_text: String, translated_text: String) -> Result<FavoriteItem, String> {
    Favorites::add(original_text, translated_text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_favorite(id: String) -> Result<(), String> {
    Favorites::delete(id).map_err(|e| e.to_string())
}
