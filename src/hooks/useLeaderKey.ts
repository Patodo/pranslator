import { useState, useEffect, useCallback, useRef } from 'react';
import { SHORTCUT_KEYS, SHORTCUT_ACTIONS } from '../config/shortcuts';

interface UseLeaderKeyOptions {
  outputText: string;
  handlers: Record<string, () => Promise<void>>;
  enabled: boolean;
}

export function useLeaderKey({ outputText, handlers, enabled }: UseLeaderKeyOptions) {
  const [isLeaderMode, setIsLeaderMode] = useState(false);
  const altPressedRef = useRef(false);
  const skipNextLeaderModeRef = useRef(false);

  const exitLeaderMode = useCallback(() => {
    setIsLeaderMode(false);
  }, []);

  useEffect(() => {
    if (!enabled) {
      setIsLeaderMode(false);
      return;
    }

    const executeShortcut = (key: string) => {
      exitLeaderMode();
      const config = SHORTCUT_ACTIONS[key];
      if (!config) return;

      if (config.requiresOutput && !outputText) return;

      const handler = handlers[config.action];
      handler?.();
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      const key = e.key.toLowerCase();

      if (e.key === 'Alt') {
        // Prevent Windows from entering menu mode
        e.preventDefault();
        altPressedRef.current = true;
      }

      if (isLeaderMode) {
        // In leader mode - handle shortcut keys
        if (SHORTCUT_KEYS.has(key)) {
          e.preventDefault();
          executeShortcut(key);
        } else if (key === 'escape') {
          exitLeaderMode();
        } else {
          // Exit on any other key (including Alt)
          skipNextLeaderModeRef.current = true;
          exitLeaderMode();
        }
      } else if (altPressedRef.current && SHORTCUT_KEYS.has(key)) {
        // Alt still held + shortcut key pressed - handle as combo
        e.preventDefault();
        skipNextLeaderModeRef.current = true;
        executeShortcut(key);
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.key === 'Alt') {
        if (skipNextLeaderModeRef.current) {
          // Skip entering leader mode after combo shortcut
          skipNextLeaderModeRef.current = false;
        } else if (!isLeaderMode) {
          // Enter leader mode on Alt release (if not already in it)
          setIsLeaderMode(true);
        }
        altPressedRef.current = false;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [enabled, isLeaderMode, outputText, handlers, exitLeaderMode]);

  return { isLeaderMode };
}
