import { useState, useEffect, useCallback, useRef } from 'react';

interface UseLeaderKeyOptions {
  outputText: string;
  handleCopy: () => Promise<void>;
  onHide: () => Promise<void>;
  enabled: boolean;
}

export function useLeaderKey({ outputText, handleCopy, onHide, enabled }: UseLeaderKeyOptions) {
  const [isLeaderMode, setIsLeaderMode] = useState(false);
  const altPressedRef = useRef(false);

  const exitLeaderMode = useCallback(() => {
    setIsLeaderMode(false);
  }, []);

  useEffect(() => {
    if (!enabled) {
      setIsLeaderMode(false);
      return;
    }

    const executeShortcut = (key: string) => {
      if (key === 'c') {
        exitLeaderMode();
        if (outputText) {
          handleCopy();
        }
      } else if (key === 'q') {
        exitLeaderMode();
        if (outputText) {
          handleCopy().then(() => {
            onHide();
          });
        } else {
          onHide();
        }
      }
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
        if (key === 'c' || key === 'q') {
          e.preventDefault();
          executeShortcut(key);
        } else if (key === 'escape') {
          exitLeaderMode();
        } else if (key !== 'alt') {
          exitLeaderMode();
        }
      } else if (altPressedRef.current && (key === 'c' || key === 'q')) {
        // Alt still held + C/Q pressed - handle as shortcut (supports Alt+C combo)
        e.preventDefault();
        executeShortcut(key);
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.key === 'Alt') {
        if (!isLeaderMode) {
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
  }, [enabled, isLeaderMode, outputText, handleCopy, onHide, exitLeaderMode]);

  return { isLeaderMode };
}
