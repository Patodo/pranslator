import type { ShortcutSettings } from '../types';

export const DEFAULT_SHORTCUTS: ShortcutSettings = {
  toggle_window: 'Alt+Shift+T',
} as const;

export const GLOBAL_SHORTCUT_CONFIGS = [
  {
    field: 'toggle_window' as const,
    name: 'Toggle Window',
    description: 'Global shortcut to show/hide the application window',
  },
] as const;
