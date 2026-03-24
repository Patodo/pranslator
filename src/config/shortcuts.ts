export interface ShortcutConfig {
  key: string;
  description: string;
  action: string;
  requiresOutput?: boolean;
}

export const SHORTCUTS: ShortcutConfig[] = [
  { key: 'd', description: 'Translate', action: 'translate' },
  { key: 'c', description: 'Copy translation', action: 'copy', requiresOutput: true },
  { key: 'q', description: 'Copy and hide', action: 'copyAndHide', requiresOutput: true },
  { key: 'b', description: 'Add to favorites', action: 'favorite', requiresOutput: true },
];

export const SHORTCUT_KEYS = new Set(SHORTCUTS.map((s) => s.key));

export const SHORTCUT_ACTIONS = SHORTCUTS.reduce(
  (acc, s) => {
    acc[s.key] = s;
    return acc;
  },
  {} as Record<string, ShortcutConfig>
);
