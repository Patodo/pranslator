import { invoke } from '@tauri-apps/api/core';
import type { TranslateRequest, TranslateResponse, Settings } from '../types';

export async function translate(request: TranslateRequest): Promise<TranslateResponse> {
  return await invoke<TranslateResponse>('translate', { request });
}

export async function getConfig(): Promise<Settings> {
  return await invoke<Settings>('get_config');
}

export async function setConfig(settings: Settings): Promise<void> {
  return await invoke<void>('set_config', { newSettings: settings });
}
