import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { DictionaryStatus, DownloadProgress } from '../types';

export async function getDictionaryStatus(): Promise<DictionaryStatus> {
  return invoke('get_dictionary_status');
}

export async function downloadDictionary(
  onProgress: (progress: DownloadProgress) => void
): Promise<void> {
  const unlisten = await listen<DownloadProgress>('dictionary-download-progress', (event) => {
    onProgress(event.payload);
  });
  try {
    await invoke('download_dictionary');
  } finally {
    unlisten();
  }
}

export async function cancelDictionaryDownload(): Promise<void> {
  return invoke('cancel_dictionary_download');
}

export async function deleteDictionary(): Promise<void> {
  return invoke('delete_dictionary');
}
