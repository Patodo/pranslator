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
  const otherKeyPressedRef = useRef(false);

  const exitLeaderMode = useCallback(() => {
    setIsLeaderMode(false);
  }, []);

  useEffect(() => {
    if (!enabled) {
      setIsLeaderMode(false);
      return;
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (isLeaderMode) {
        const key = e.key.toLowerCase();

        if (key === 'c') {
          e.preventDefault();
          exitLeaderMode();
          if (outputText) {
            handleCopy();
          }
        } else if (key === 'q') {
          e.preventDefault();
          exitLeaderMode();
          if (outputText) {
            handleCopy().then(() => {
              onHide();
            });
          } else {
            onHide();
          }
        } else if (key === 'escape') {
          exitLeaderMode();
        } else if (key !== 'alt') {
          exitLeaderMode();
        }
      } else {
        if (e.key === 'Alt') {
          // Prevent Windows from entering menu mode (which swallows subsequent keydown)
          e.preventDefault();
          altPressedRef.current = true;
          otherKeyPressedRef.current = false;
        } else if (altPressedRef.current) {
          otherKeyPressedRef.current = true;
        }
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.key === 'Alt' && altPressedRef.current) {
        if (!isLeaderMode && !otherKeyPressedRef.current) {
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
