import { useState, useEffect, useCallback, useRef } from 'react';
import { Keyboard } from 'lucide-react';
import { validateShortcut } from '../api/translate';

interface ShortcutInputProps {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}

export function ShortcutInput({ value, onChange, disabled }: ShortcutInputProps) {
  const [isListening, setIsListening] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLButtonElement>(null);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!isListening) return;

      e.preventDefault();
      e.stopPropagation();

      // Escape cancels recording
      if (e.key === 'Escape') {
        setIsListening(false);
        return;
      }

      // Ignore standalone modifier keys
      if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) {
        return;
      }

      // Collect modifiers
      const modifiers: string[] = [];
      if (e.ctrlKey) modifiers.push('Ctrl');
      if (e.altKey) modifiers.push('Alt');
      if (e.shiftKey) modifiers.push('Shift');

      // Need at least one modifier + a key
      if (modifiers.length === 0) {
        return;
      }

      // Build shortcut string - use e.code for physical key to avoid Shift+2 -> @ issue
      let key = e.code;
      // Handle digit keys (Digit1 -> 1, Digit2 -> 2, etc.)
      if (key.startsWith('Digit')) {
        key = key.slice(5);
      }
      // Handle numpad keys (Numpad1 -> Num1, etc.)
      if (key.startsWith('Numpad')) {
        key = 'Num' + key.slice(6);
      }
      // Handle special keys
      if (key === 'Space') key = 'Space';
      if (key === 'ArrowUp') key = 'Up';
      if (key === 'ArrowDown') key = 'Down';
      if (key === 'ArrowLeft') key = 'Left';
      if (key === 'ArrowRight') key = 'Right';
      // For letter keys, KeyA -> A, KeyB -> B, etc.
      if (key.startsWith('Key') && key.length === 4) {
        key = key.slice(3);
      }

      const shortcut = [...modifiers, key].join('+');

      // Validate and apply
      validateShortcut(shortcut)
        .then(() => {
          onChange(shortcut);
          setError(null);
          setIsListening(false);
        })
        .catch((err) => {
          setError(err);
        });
    },
    [isListening, onChange]
  );

  useEffect(() => {
    if (isListening) {
      window.addEventListener('keydown', handleKeyDown, true);
      return () => window.removeEventListener('keydown', handleKeyDown, true);
    }
  }, [isListening, handleKeyDown]);

  const handleClick = () => {
    setError(null);
    setIsListening(true);
    inputRef.current?.focus();
  };

  const handleBlur = () => {
    if (isListening) {
      setIsListening(false);
    }
  };

  return (
    <div className="shortcut-input-wrapper">
      <button
        ref={inputRef}
        className={`shortcut-record-btn ${isListening ? 'recording' : ''}`}
        onClick={handleClick}
        onBlur={handleBlur}
        disabled={disabled}
        type="button"
      >
        <Keyboard size={14} />
        <span>{isListening ? '按下快捷键...' : value || '点击设置快捷键'}</span>
      </button>
      {error && <span className="shortcut-error">{error}</span>}
    </div>
  );
}
