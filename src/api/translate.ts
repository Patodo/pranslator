import { invoke } from '@tauri-apps/api/core';
import type { TranslateRequest, TranslateResponse, Settings, FavoriteItem } from '../types';

export async function translate(request: TranslateRequest): Promise<TranslateResponse> {
  return await invoke<TranslateResponse>('translate', { request });
}

export async function getConfig(): Promise<Settings> {
  return await invoke<Settings>('get_config');
}

export async function setConfig(settings: Settings): Promise<void> {
  return await invoke<void>('set_config', { newSettings: settings });
}

export async function validateShortcut(shortcut: string): Promise<void> {
  return await invoke<void>('validate_shortcut', { shortcut });
}

export async function hideWindow(): Promise<void> {
  return await invoke<void>('hide_window');
}

export async function getFavorites(): Promise<FavoriteItem[]> {
  return await invoke<FavoriteItem[]>('get_favorites');
}

export async function addFavorite(
  originalText: string,
  translatedText: string
): Promise<FavoriteItem> {
  return await invoke<FavoriteItem>('add_favorite', { originalText, translatedText });
}

export async function deleteFavorite(id: string): Promise<void> {
  return await invoke<void>('delete_favorite', { id });
}
