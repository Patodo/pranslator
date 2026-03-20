import { create } from 'zustand';
import type { Settings } from '../types';
import { getConfig, setConfig } from '../api/translate';

interface SettingsState {
  settings: Settings | null;
  isLoading: boolean;
  error: string | null;
  loadSettings: () => Promise<void>;
  updateSettings: (settings: Settings) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: null,
  isLoading: false,
  error: null,

  loadSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      const settings = await getConfig();
      set({ settings, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  updateSettings: async (newSettings: Settings) => {
    try {
      await setConfig(newSettings);
      set({ settings: newSettings });
    } catch (error) {
      set({ error: String(error) });
    }
  },
}));
